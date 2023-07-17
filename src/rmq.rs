use core::num;
use std::path::Path;

#[derive(MallocSizeOf)]
pub struct RMQNaive {
    naive: Vec<Vec<u64>>,
}

impl RMQNaive {
    pub fn new(numbers: &Vec<u64>) -> Self {
        numbers.sort();

        let u = numbers[numbers.len() - 1];

        // O(u^2). Could do u log n using consecutively less space for "to".
        //
        let mut naive = vec![vec![0; u + 1]; u + 1];

        return Self { naive: naive };
    }

    pub fn select(&self, from: usize, to: usize) -> u64 {
        return self.naive[from][to];
    }
}

pub fn rmq(path: &Path) {
    println!("rmq");
}

#[test]
fn testing_pd_test() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_4.txt");

    let want = vec![0, 0, 2, 2, 4, 4, 4, 7, 7, 7, 7];

    benchmark_and_check(path, Some(want));
}

#[test]
fn testing_pd_benchmark1() {
    let path = Path::new("testdata/predecessor_examples/predecessor_example_1.txt");

    benchmark_and_check(path, None);
}
