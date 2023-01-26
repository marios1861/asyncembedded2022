use json::Value;
use serde::Serialize;
use serde_json as json;
use std::collections::VecDeque;
use tokio::fs::File;
use tokio::io;
use tokio::io::AsyncWriteExt;

#[derive(Serialize)]
struct MovingAverageData {
    mav_avg: f64,
    tot_vol: i64,
}

pub struct MovingAverage {
    mav_vals: VecDeque<f64>,
    mav_avg: f64,
    vols: VecDeque<i64>,
    capacity: u64,
}

impl MovingAverage {
    pub fn new(capacity: u64) -> MovingAverage {
        MovingAverage {
            mav_vals: VecDeque::with_capacity(capacity.try_into().unwrap()),
            mav_avg: 0.0,
            vols: VecDeque::with_capacity(capacity.try_into().unwrap()),
            capacity,
        }
    }

    pub fn is_full(&self) -> bool {
        self.mav_vals.capacity() == self.mav_vals.len()
    }

    fn price_avg(data: &Vec<&Value>) -> f64 {
        let prices_sum: f64 = data
            .iter()
            .filter_map(|val| val.as_object()?.get("p")?.as_f64())
            .sum();
        prices_sum / (data.len() as f64)
    }

    fn vol_sum(data: &Vec<&Value>) -> i64 {
        data.iter()
            .filter_map(|val| val.as_object()?.get("v")?.as_i64())
            .sum()
    }

    pub fn init_update(&mut self, data: &Vec<&Value>) {
        let new_avg = Self::price_avg(data);
        self.mav_vals.push_front(new_avg);

        self.mav_avg = new_avg;

        self.vols.push_front(Self::vol_sum(data));
    }

    pub fn update(&mut self, data: &Vec<&Value>) -> Option<()> {
        let new_avg = Self::price_avg(data);
        self.mav_vals.push_front(new_avg);
        let last_val = self.mav_vals.front()?;
        self.mav_avg = self.mav_avg + (new_avg - last_val) / self.capacity as f64;
        self.vols.push_front(Self::vol_sum(data));
        Some(())
    }

    fn get_writable(&self) -> MovingAverageData {
        MovingAverageData {
            mav_avg: self.mav_avg,
            tot_vol: Iterator::sum(self.vols.iter()),
        }
    }

    pub async fn save(&self, file: &mut File) -> io::Result<()> {
        file.write_all(json::to_string(&Self::get_writable(&self))?.as_bytes())
            .await
    }
}
