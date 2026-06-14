use std::collections::HashSet;

use ratatui::widgets::TableState;

use crate::simctl::{Simulator, Udid};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sort {
    SizeDesc,
    Name,
    Runtime,
}

impl Sort {
    pub fn label(self) -> &'static str {
        match self {
            Self::SizeDesc => "size",
            Self::Name => "name",
            Self::Runtime => "runtime",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::SizeDesc => Self::Name,
            Self::Name => Self::Runtime,
            Self::Runtime => Self::SizeDesc,
        }
    }
}

pub enum Modal {
    None,
    Confirm,
    Deleting,
    Error(String),
}

pub struct App {
    pub sims: Vec<Simulator>,
    pub table: TableState,
    pub selected: HashSet<Udid>,
    pub sort: Sort,
    pub modal: Modal,
    pub scanning: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new(sims: Vec<Simulator>) -> Self {
        let mut table = TableState::default();
        if !sims.is_empty() {
            table.select(Some(0));
        }
        let mut app = Self {
            sims,
            table,
            selected: HashSet::new(),
            sort: Sort::SizeDesc,
            modal: Modal::None,
            scanning: true,
            should_quit: false,
        };
        app.resort();
        app
    }

    pub fn resort(&mut self) {
        let focused_udid = self.focused_udid();
        match self.sort {
            Sort::SizeDesc => self.sims.sort_by(|a, b| {
                b.size_bytes
                    .unwrap_or(0)
                    .cmp(&a.size_bytes.unwrap_or(0))
                    .then_with(|| a.name.cmp(&b.name))
            }),
            Sort::Name => self.sims.sort_by(|a, b| a.name.cmp(&b.name)),
            Sort::Runtime => self
                .sims
                .sort_by(|a, b| a.runtime.cmp(&b.runtime).then_with(|| a.name.cmp(&b.name))),
        }
        if let Some(udid) = focused_udid
            && let Some(idx) = self.sims.iter().position(|s| s.udid == udid)
        {
            self.table.select(Some(idx));
        }
    }

    pub fn focused_udid(&self) -> Option<Udid> {
        self.table
            .selected()
            .and_then(|i| self.sims.get(i))
            .map(|s| s.udid.clone())
    }

    pub fn move_cursor(&mut self, delta: isize) {
        if self.sims.is_empty() {
            return;
        }
        let len = self.sims.len() as isize;
        let cur = self.table.selected().unwrap_or(0) as isize;
        let next = (cur + delta).rem_euclid(len);
        self.table.select(Some(next as usize));
    }

    pub fn toggle_select(&mut self) {
        if let Some(udid) = self.focused_udid()
            && !self.selected.remove(&udid)
        {
            self.selected.insert(udid);
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort = self.sort.next();
        self.resort();
    }

    pub fn apply_size(&mut self, udid: &str, bytes: u64) {
        if let Some(sim) = self.sims.iter_mut().find(|s| s.udid == udid) {
            sim.size_bytes = Some(bytes);
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.sims.iter().filter_map(|s| s.size_bytes).sum()
    }

    pub fn selected_bytes(&self) -> u64 {
        self.sims
            .iter()
            .filter(|s| self.selected.contains(&s.udid))
            .filter_map(|s| s.size_bytes)
            .sum()
    }

    pub fn selected_sims(&self) -> Vec<&Simulator> {
        self.sims
            .iter()
            .filter(|s| self.selected.contains(&s.udid))
            .collect()
    }

    pub fn remove_deleted(&mut self, udids: &[Udid]) {
        let set: HashSet<&str> = udids.iter().map(|s| s.as_str()).collect();
        self.sims.retain(|s| !set.contains(s.udid.as_str()));
        self.selected.retain(|u| !set.contains(u.as_str()));
        if self.sims.is_empty() {
            self.table.select(None);
        } else if self.table.selected().is_none_or(|i| i >= self.sims.len()) {
            self.table.select(Some(self.sims.len() - 1));
        }
    }
}
