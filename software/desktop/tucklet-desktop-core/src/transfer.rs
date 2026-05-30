// transfer.rs
// Desktop transfer engine: the honest pre-transfer estimate (from the shared,
// tested tucklet-core estimator) plus a live ETA recomputed from measured
// throughput as a batch runs. The per-item work is supplied by the caller
// (upload via DataClient for offload, download for load).
//
// License: PolyForm Noncommercial 1.0.0

use std::time::Instant;
use tucklet_core::estimate::{self, Estimate};
use tucklet_core::link::LinkProfile;
use tucklet_proto::TransferItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub items_total: u32,
    pub items_done: u32,
    pub bytes_total: u64,
    pub bytes_done: u64,
    pub eta_seconds: u32,
    pub throughput_bps: u64,
}

pub struct TransferEngine {
    link: LinkProfile,
}

impl TransferEngine {
    pub fn new(link: LinkProfile) -> Self {
        Self { link }
    }

    /// The number we show before anything moves.
    pub fn estimate(&self, items: &[TransferItem]) -> Estimate {
        let sizes: Vec<u64> = items.iter().map(|i| i.size_bytes).collect();
        estimate::estimate_sizes(&sizes, self.link)
    }

    /// Run a batch, calling `per_item` for each item and reporting progress with
    /// a live ETA via `progress_cb`. Returns the ids that completed (for offload
    /// the caller then deletes the local originals + offers undo).
    pub fn run_batch<W, P>(
        &self,
        items: &[TransferItem],
        mut per_item: W,
        mut progress_cb: P,
    ) -> Vec<String>
    where
        W: FnMut(&TransferItem) -> Result<(), String>,
        P: FnMut(Progress),
    {
        let bytes_total: u64 = items.iter().map(|i| i.size_bytes).sum();
        let mut bytes_done = 0u64;
        let mut completed = Vec::new();
        let start = Instant::now();

        // Seed with the static estimate so the UI shows a number immediately.
        let seed = self.estimate(items);
        progress_cb(Progress {
            items_total: items.len() as u32,
            items_done: 0,
            bytes_total,
            bytes_done: 0,
            eta_seconds: seed.seconds,
            throughput_bps: self.link.sustained_bps,
        });

        for (idx, item) in items.iter().enumerate() {
            if per_item(item).is_err() {
                break;
            }
            completed.push(item.id.clone());
            bytes_done += item.size_bytes;
            let elapsed = start.elapsed().as_secs_f64().max(0.001);
            let measured_bps = (bytes_done as f64 / elapsed) as u64;
            let remaining = bytes_total - bytes_done;
            progress_cb(Progress {
                items_total: items.len() as u32,
                items_done: (idx + 1) as u32,
                bytes_total,
                bytes_done,
                eta_seconds: estimate::eta_seconds(remaining, measured_bps.max(1)),
                throughput_bps: measured_bps,
            });
        }
        completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tucklet_core::link;

    fn items(n: usize, size: u64) -> Vec<TransferItem> {
        (0..n)
            .map(|i| TransferItem {
                id: format!("id{i}"),
                size_bytes: size,
                mime: "image/heic".into(),
            })
            .collect()
    }

    #[test]
    fn estimate_matches_core() {
        let e = TransferEngine::new(link::C5_WIRELESS);
        let est = e.estimate(&items(30, 4_000_000));
        // Same expectation as the core test: 30 x 4MB over C5 ~ 15s.
        assert!((14..=16).contains(&est.seconds), "got {}", est.seconds);
        assert_eq!(est.files, 30);
    }

    #[test]
    fn run_batch_reports_completion_and_progress() {
        let e = TransferEngine::new(link::C5_WIRELESS);
        let its = items(3, 1_000_000);
        let mut last: Option<Progress> = None;
        let done = e.run_batch(&its, |_| Ok(()), |p| last = Some(p));
        assert_eq!(done.len(), 3);
        let p = last.unwrap();
        assert_eq!(p.items_done, 3);
        assert_eq!(p.bytes_done, 3_000_000);
    }

    #[test]
    fn run_batch_stops_on_error() {
        let e = TransferEngine::new(link::C5_WIRELESS);
        let its = items(5, 1_000_000);
        let mut count = 0;
        let done = e.run_batch(
            &its,
            |_| {
                count += 1;
                if count >= 3 {
                    Err("link dropped".into())
                } else {
                    Ok(())
                }
            },
            |_| {},
        );
        assert_eq!(done.len(), 2); // two succeeded before the third failed
    }
}
