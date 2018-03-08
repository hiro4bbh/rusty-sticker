#![allow(non_snake_case)]

use std::collections::{HashMap,HashSet};
use std::env;
use std::fs::File;
use std::hash;
use std::io::{BufRead,BufReader};
use std::path::Path;
use std::process;
use std::time::Instant;

extern crate getopts;
use getopts::{Matches,Options};

extern crate time;

#[macro_use] extern crate rusty_sticker;

type Hasher = hash::BuildHasherDefault<rusty_sticker::hash::Hasher>;

type FeatureVector = Vec<(u32, f32)>;
type FeatureVectors = Vec<FeatureVector>;

type LabelVector = Vec<u32>;
type LabelVectors = Vec<LabelVector>;

struct Dataset {
    X: FeatureVectors,
    Y: LabelVectors
}

impl Dataset {
    fn resize(&mut self, n: usize) {
        self.X.resize(n, vec![]);
        self.Y.resize(n, vec![]);
    }
    fn size(&self) -> usize {
        return self.X.len()
    }
}

impl<'a> IntoIterator for &'a Dataset {
    type Item = (&'a FeatureVector, &'a LabelVector);
    type IntoIter = DatasetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        return DatasetIterator{ ds: self, index: 0 }
    }
}

struct DatasetIterator<'a> {
    ds: &'a Dataset,
    index: usize
}

impl<'a> Iterator for DatasetIterator<'a> {
    type Item = (&'a FeatureVector, &'a LabelVector);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index >= self.ds.size() {
            return None;
        }
        self.index += 1;
        return Some((&self.ds.X[index], &self.ds.Y[index]));
    }
}

fn read_dataset<P: AsRef<Path>>(filename: P) -> Dataset {
    let mut ds = Dataset{
        X: FeatureVectors::new(),
        Y: LabelVectors::new()
    };
    let f = File::open(filename).unwrap();
    let file = BufReader::new(&f);
    let mut lines = file.lines();
    lines.next();
    for line in lines {
        let line = line.unwrap();
        let mut words = line.split(" ");
        let labels = words.next().unwrap().split(",");
        let labels = labels.map(|s| { s.parse().unwrap() }).collect::<LabelVector>();
        ds.Y.push(labels);
        let features = words.map(|s| {
            let mut parts = s.split(":");
            let key: u32 = parts.next().unwrap().parse().unwrap();
            let value: f32 = parts.next().unwrap().parse().unwrap();
            (key, value)
        }).collect::<FeatureVector>();
        ds.X.push(features);
    }
    return ds;
}

struct DatasetIndex<'a> {
    nfeatures_list: Vec<u32>,
    indices: HashMap<u32, Vec<(u32, f32)>, Hasher>,
    labelvecs: &'a LabelVectors,
}

type DatasetIndexContext = Vec<(f32, u32)>;

impl<'a> DatasetIndex<'a> {
    fn new(ds: &Dataset) -> DatasetIndex {
        let mut indices: HashMap<u32, Vec<(u32, f32)>, Hasher> = HashMap::default();
        let mut nfeatures_list = vec![0u32; ds.X.len()];
        for (i, (xi, _)) in ds.into_iter().enumerate() {
            let mut xinorm = 0.0f32;
            for &(_, value) in xi {
                xinorm += value*value;
            }
            xinorm = xinorm.sqrt();
            for &(key, value) in xi {
                indices.entry(key).or_insert(vec![]).push((i as u32, value/xinorm));
            }
            nfeatures_list[i] = xi.len() as u32;
        }
        return DatasetIndex{
            nfeatures_list: nfeatures_list,
            indices: indices,
            labelvecs: &ds.Y
        };
    }

    fn find_nearests(&self, xi: &FeatureVector, S: usize, beta: f32, ctx: &mut DatasetIndexContext) -> Vec<(u32, f32)> {
        let sim_counts = ctx;
        for &(key, value) in xi {
            match self.indices.get(&key) {
                Some(index) => { unsafe { for &(i, v) in index {
                    let p = sim_counts.get_unchecked_mut(i as usize);
                    (*p).0 += value * v;
                    (*p).1 += 1;
                } } },
                None => {}
            }
        }
        let mut index_sims: Vec<(u32, f32)> = Vec::with_capacity(S);
        for (i, &mut (ref mut psim, ref mut pcount)) in sim_counts.iter_mut().enumerate() {
            if *pcount > 0 {
                if *psim > 0.0 {
                    let jaccard = (*pcount as f32)/((self.nfeatures_list[i] + (xi.len() as u32) - *pcount) as f32);
                    let mut sim = *psim;
                    if beta == 0.0 {
                    } else if beta == 1.0 {
                        sim *= jaccard;
                    } else {
                        sim *= jaccard.powf(beta);
                    }
                    if index_sims.len() == 0 {
                        index_sims.push((i as u32, sim));
                    } else if index_sims.last().unwrap().1 > sim {
                        if index_sims.len() < S {
                            index_sims.push((i as u32, sim));
                        }
                    } else {
                        for k in 0..(index_sims.len()) {
                            if sim >= index_sims[k].1 {
                                if index_sims.len() < S {
                                    index_sims.push((0, 0.0f32));
                                }
                                for l in (k..(index_sims.len()-1)).rev() {
                                    index_sims[l+1] = index_sims[l];
                                }
                                index_sims[k] = (i as u32, sim);
                                break;
                            }
                        }
                    }
                }
                *psim = 0.0f32;
                *pcount = 0;
            }
        }
        return index_sims;
    }

    fn new_context(&self) -> DatasetIndexContext {
        return vec![(0.0f32, 0); self.labelvecs.len()];
    }
}

fn run_test(index: &DatasetIndex, ds: &Dataset, K: usize, S: usize, alpha: f32, beta: f32, per: usize) -> LabelVectors {
    let mut yhat: LabelVectors = Vec::with_capacity(ds.size());
    let mut ctx = index.new_context();
    for (i, (xi, _)) in ds.into_iter().enumerate() {
        let mut xinorm = 0.0f32;
        for &(_, value) in xi {
            xinorm += value*value;
        }
        xinorm = xinorm.sqrt();
        let index_sims = index.find_nearests(xi, S, beta, &mut ctx);
        let mut label_hist: HashMap<u32, f32, Hasher> = HashMap::default();
        for &(j, sim) in &index_sims {
            let sim = (sim/xinorm).powf(alpha);
            for &label in &index.labelvecs[j as usize] {
                *label_hist.entry(label).or_insert(0.0f32) += sim
            }
        }
        let mut labels_topK: Vec<(u32, f32)> = Vec::new();
        for (label, freq) in label_hist {
            if labels_topK.len() == 0 {
                labels_topK.push((label, freq));
            } else if labels_topK.last().unwrap().1 > freq || (labels_topK.last().unwrap().1 == freq && labels_topK.last().unwrap().0 < label) {
                if labels_topK.len() < K {
                    labels_topK.push((label, freq));
                }
            } else {
                for k in 0..(labels_topK.len()) {
                    if freq > labels_topK[k].1 || (freq == labels_topK[k].1 && label <= labels_topK[k].0) {
                        if labels_topK.len() < K {
                            labels_topK.push((0, 0.0f32));
                        }
                        for l in (k..(labels_topK.len()-1)).rev() {
                            labels_topK[l+1] = labels_topK[l];
                        }
                        labels_topK[k] = (label, freq);
                        break;
                    }
                }
            }
        }
        let mut yihat: LabelVector = Vec::with_capacity(labels_topK.len());
        for &(label, _) in &labels_topK {
            yihat.push(label);
        }
        yhat.push(yihat);
        if per > 0 && i%per == 0 {
            println!("i={}: ", i);
            println!("    {:?}", index_sims);
            println!("    -> {:?}", labels_topK);
            println!("       <> {:?}", ds.Y[i]);
        }
    }
    return yhat;
}

fn report_precision(Yhat: &LabelVectors, Y: &LabelVectors, K: usize) -> (f32, f32) {
    let mut sumPK = 0.0f32;
    for (i, yihat) in Yhat.iter().enumerate() {
        let yi = &Y[i];
        let mut yimap: HashSet<u32, Hasher> = HashSet::default();
        for label in yi {
            yimap.insert(*label);
        }
        let mut pKi = 0;
        for k in 0..(yihat.len().min(K)) {
            if yimap.contains(&yihat[k]) {
                pKi += 1;
            }
        }
        sumPK += (pKi as f32)/(K as f32);
    }
    let avgPK = sumPK/(Yhat.len() as f32);
    let mut sumMaxPK = 0.0f32;
    for yi in Y {
        sumMaxPK += (yi.len().min(K) as f32)/(K as f32);
    }
    let avgMaxPK = sumMaxPK/(Y.len() as f32);
    return (avgPK, avgMaxPK);
}

fn run(optvals: Matches) {
    let mut Ks = optvals.opt_strs("K");
    if Ks.len() == 0 {
        Ks = vec![String::from("1"), String::from("3"), String::from("5")];
    }
    let Ks: Vec<usize> = Ks.iter().map(|K| { match K.parse::<usize>() {
        Ok(K) => { K },
        Err(e) => panic!("illegal K: {}", e.to_string())
    }}).collect();
    let maxK = *Ks.iter().max().unwrap() as usize;
    let alpha = match optvals.opt_str("alpha").unwrap_or(String::from("1.0")).parse::<f32>() {
        Ok(alpha) => { alpha },
        Err(e) => panic!("illegal alpha: {}", e.to_string())
    };
    let beta = match optvals.opt_str("beta").unwrap_or(String::from("0.0")).parse::<f32>() {
        Ok(beta) => { beta },
        Err(e) => panic!("illegal beta: {}", e.to_string())
    };
    let N = match optvals.opt_str("N").unwrap_or(String::from("-1")).parse::<isize>() {
        Ok(N) => { N },
        Err(e) => panic!("illegal N: {}", e.to_string())
    };
    let per = match optvals.opt_str("per").unwrap_or(String::from("0")).parse::<usize>() {
        Ok(per) => { per },
        Err(e) => panic!("illegal per: {}", e.to_string())
    };
    let S = match optvals.opt_str("S").unwrap_or(String::from("5")).parse::<usize>() {
        Ok(S) => { S },
        Err(e) => panic!("illegal S: {}", e.to_string())
    };
    if optvals.free.len() < 1 {
        panic!("specify dataset root path");
    }
    let dsroot = &optvals.free[0];

    let train_ds_path = Path::new(dsroot).join("train.txt");
    info!("reading training table from {:?}", train_ds_path);
    let train_ds = read_dataset(train_ds_path);
    info!("read training table with {} entries", train_ds.size());
    let test_ds_path = Path::new(dsroot).join("test.txt");
    info!("reading test table from {:?}", test_ds_path);
    let mut test_ds = read_dataset(test_ds_path);
    info!("read test table with {} entries", test_ds.size());

    info!("constructing training set index ...");
    let start_time = Instant::now();
    let train_index = DatasetIndex::new(&train_ds);
    let t = start_time.elapsed();
    info!("finished training set index construction in {}.{:03}s", t.as_secs(), t.subsec_nanos()/1_000_000);

    info!("starting top-{} inference of {} entries with hyper-parameters S={},alpha={},beta={} ...", maxK, test_ds.size(), S, alpha, beta);
    let start_time = Instant::now();
    let N = if N < 0 || N > (test_ds.size() as isize) { test_ds.size() as usize } else { N as usize };
    test_ds.resize(N);
    let yhat = run_test(&train_index, &test_ds, maxK, S, alpha, beta, per);
    let t = start_time.elapsed();
    let t_per_entry = t.checked_div(yhat.len() as u32).unwrap();
    info!("finished inference of {} entries in {}.{:03}s ({:.03}ms/entry)", yhat.len(), t.as_secs(), t.subsec_nanos()/1_000_000, (t_per_entry.subsec_nanos() as f32)/1_000_000.0f32);
    for &K in &Ks {
        let (avgPK, avgMaxPK) = report_precision(&yhat, &test_ds.Y, K);
        println!("Precision@{}={:5.2}/{:5.2}%", K, avgPK*100.0, avgMaxPK*100.0);
    }
    info!("finished rusty-sticker");
}

fn show_help(progname: &str, opts: Options) {
    println!("rusty-sticker");
    println!("Copyright 2018- Tatsuhiro Aoshima (hiro4bbh@gmail.com).");
    print!("{}", opts.usage(&format!("Usage: {} [options] dataset-root", progname)));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let progname = &args[0];
    let mut opts = Options::new();
    opts.optopt("", "alpha", "specify the smoothing parameter of similarities", "VALUE");
    opts.optopt("", "beta", "specify the balancing parameter of the Jaccard and cosine similarity", "VALUE");
    opts.optflag("h", "help", "show the help and exit");
    opts.optmulti("K", "", "specify the values of top-K", "VALUE");
    opts.optopt("N", "", "specify the maximum number of the tested data entries", "VALUE");
    opts.optopt("", "per", "specify the prediction inspection interval", "VALUE");
    opts.optopt("S", "", "specify the size of neighborhood", "VALUE");
    let optvals = match opts.parse(&args[1..]) {
        Ok(optvals) => { optvals },
        Err(e) => { panic!(e.to_string()) }
    };
    if optvals.opt_present("h") {
        show_help(progname, opts);
        process::exit(1);
    }
    run(optvals);
}
