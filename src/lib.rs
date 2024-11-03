use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Write},
    sync::{Mutex, MutexGuard},
};

const QUEUE_FILE_PATH: &'static str = "/home/fuad1502/queue.txt";
const OLD_QUEUE_FILE_PATH: &'static str = "/home/fuad1502/queue.txt.old";

pub struct Queue {
    file_queue_pair: Mutex<(Option<fs::File>, VecDeque<String>)>,
}

impl Queue {
    pub fn new() -> Result<Self, String> {
        let file = Self::open_queue_file()?;
        let queue = Self::get_queue_from_file(&file)?;
        let file_queue_pair = Mutex::new((Some(file), queue));
        Ok(Queue { file_queue_pair })
    }

    pub fn add(&mut self, item: &str) -> Result<(), String> {
        let mut guard = self.get_file_queue_pair_guard()?;
        if let Some(file) = &mut guard.0 {
            Self::write_item_to_file(item, file)?;
            let queue = &mut guard.1;
            queue.push_back(item.to_string());
            return Ok(());
        }
        Err("".to_string())
    }

    pub fn remove(&mut self, item: &str) -> Result<String, String> {
        let mut guard = self.get_file_queue_pair_guard()?;
        let queue = &mut guard.1;
        if let Some(idx) = Self::get_item_idx_in_queue(item, queue) {
            let item = queue.remove(idx).expect("Reached unexpected path.");
            Self::rewrite_file_with_queue(guard)?;
            return Ok(item);
        }
        Err("".to_string())
    }

    pub fn pop(&mut self) -> Result<String, String> {
        let mut guard = self.get_file_queue_pair_guard()?;
        let queue = &mut guard.1;
        if let Some(item) = queue.pop_front() {
            Self::rewrite_file_with_queue(guard)?;
            return Ok(item);
        };
        Err("".to_string())
    }

    fn open_queue_file() -> Result<fs::File, String> {
        let file = Self::open_existing_file(QUEUE_FILE_PATH);
        let old_file = Self::open_existing_file(OLD_QUEUE_FILE_PATH);
        match (file, old_file) {
            (Ok(_), Ok(_)) => {
                Err("Both {FILE_PATH} and {FILE_PATH_OLD} exists, possible corruption detected, resolve issue manually.".to_string())
            }
            (Err(_), Ok(file)) => {
                fs::rename(OLD_QUEUE_FILE_PATH, QUEUE_FILE_PATH).expect("IO error.");
                Ok(file)
            }
            (Ok(file), Err(_)) => Ok(file),
            (Err(_), Err(_)) => {
                let file = Self::create_new_file(QUEUE_FILE_PATH)?;
                Ok(file)
            }
        }
    }

    fn get_queue_from_file(file: &fs::File) -> Result<VecDeque<String>, String> {
        let mut queue = VecDeque::new();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            match line {
                Ok(line) => queue.push_back(line),
                Err(e) => return Err(e.to_string()),
            }
        }
        Ok(queue)
    }

    fn get_item_idx_in_queue(item: &str, queue: &VecDeque<String>) -> Option<usize> {
        match queue.iter().enumerate().find(|&i| i.1 == item) {
            Some((idx, _)) => Some(idx),
            None => None,
        }
    }

    fn get_file_queue_pair_guard(
        &mut self,
    ) -> Result<MutexGuard<(Option<fs::File>, VecDeque<String>)>, String> {
        match self.file_queue_pair.lock() {
            Ok(r) => Ok(r),
            Err(e) => Err(e.to_string()),
        }
    }

    fn write_item_to_file(item: &str, file: &mut fs::File) -> Result<(), String> {
        match writeln!(file, "{item}") {
            Ok(_) => Ok(()),
            Err(e) => return Err(e.to_string()),
        }
    }

    fn rewrite_file_with_queue(
        mut guard: MutexGuard<(Option<fs::File>, VecDeque<String>)>,
    ) -> Result<(), String> {
        fs::rename(QUEUE_FILE_PATH, OLD_QUEUE_FILE_PATH).expect("IO error.");
        let mut new_file = Self::create_new_file(QUEUE_FILE_PATH)?;
        let queue = &guard.1;
        for item in queue {
            Self::write_item_to_file(item, &mut new_file)?;
        }
        let current_file = &mut guard.0;
        _ = current_file.insert(new_file);
        fs::remove_file(OLD_QUEUE_FILE_PATH).expect("IO error.");
        Ok(())
    }

    fn open_existing_file(path: &str) -> Result<fs::File, String> {
        match fs::OpenOptions::new().append(true).read(true).open(path) {
            Ok(file) => Ok(file),
            Err(e) => Err(e.to_string()),
        }
    }

    fn create_new_file(path: &str) -> Result<fs::File, String> {
        match fs::OpenOptions::new()
            .append(true)
            .read(true)
            .create_new(true)
            .open(path)
        {
            Ok(file) => Ok(file),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn all() {
        let mut queue = Queue::new().unwrap();
        queue.add("fuad").unwrap();
        queue.add("ismail").unwrap();
        queue.add("faisal").unwrap();
        queue.add("ibrahim").unwrap();
        queue.remove("ismail").unwrap();
        queue.pop().unwrap();
        queue.pop().unwrap();
    }
}
