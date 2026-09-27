use rand::seq::SliceRandom;
use rand::Rng;
use std::time::Instant;

fn quicksort_single(arr: &mut [i64]) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    let mut rng = rand::rng();
    arr.shuffle(&mut rng);
    quicksort_single_rec(arr, 0, (n - 1) as isize);
}

fn quicksort_single_rec(arr: &mut [i64], low: isize, high: isize) {
    if low >= high {
        return;
    }

    let pivot = arr[low as usize];

    // 3-veis partisjonering: arr[low..lt) < pivot, arr[lt..=gt] == pivot,
    // arr[gt+1..=high] > pivot.
    let mut lt = low;
    let mut gt = high;
    let mut i = low + 1;
    while i <= gt {
        if arr[i as usize] < pivot {
            arr.swap(lt as usize, i as usize);
            lt += 1;
            i += 1;
        } else if arr[i as usize] > pivot {
            arr.swap(i as usize, gt as usize);
            gt -= 1;
        } else {
            i += 1;
        }
    }

    quicksort_single_rec(arr, low, lt - 1);
    quicksort_single_rec(arr, gt + 1, high);
}

// Dual-pivot quicksort
// https://www.geeksforgeeks.org/dual-pivot-quicksort/
fn quicksort_dual(arr: &mut [i64]) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    quicksort_dual_rec(arr, 0, (n - 1) as isize);
}

fn quicksort_dual_rec(arr: &mut [i64], low: isize, high: isize) {
    if low >= high {
        return;
    }

    // Fiks 1: unngå skjevdeling på tabeller som er sortert fra før.
    let third = (high - low) / 3;
    arr.swap(low as usize, (low + third) as usize);
    arr.swap(high as usize, (high - third) as usize);

    if arr[low as usize] > arr[high as usize] {
        arr.swap(low as usize, high as usize);
    }

    let pivot1 = arr[low as usize];
    let pivot2 = arr[high as usize];

    let mut lt = low + 1; // grense: arr[low+1..lt) < pivot1
    let mut gt = high - 1; // grense: arr[gt+1..high] > pivot2
    let mut i = low + 1;

    while i <= gt {
        if arr[i as usize] < pivot1 {
            arr.swap(i as usize, lt as usize);
            lt += 1;
        } else if arr[i as usize] >= pivot2 {
            while arr[gt as usize] > pivot2 && i < gt {
                gt -= 1;
            }
            arr.swap(i as usize, gt as usize);
            gt -= 1;
            if arr[i as usize] < pivot1 {
                arr.swap(i as usize, lt as usize);
                lt += 1;
            }
        }
        i += 1;
    }
    lt -= 1;
    gt += 1;

    arr.swap(low as usize, lt as usize);
    arr.swap(high as usize, gt as usize);

    quicksort_dual_rec(arr, low, lt - 1);
    // Fiks 2: hopp over midtintervallet hvis de to pivotene er like.
    if pivot1 != pivot2 {
        quicksort_dual_rec(arr, lt + 1, gt - 1);
    }
    quicksort_dual_rec(arr, gt + 1, high);
}


// Tester
fn checksum(arr: &[i64]) -> i64 {
    arr.iter().sum()
}

fn is_sorted(arr: &[i64]) -> bool {
    for i in 0..arr.len().saturating_sub(1) {
        if arr[i + 1] < arr[i] {
            return false;
        }
    }
    true
}


// Datagenerering
fn generate_random(n: usize) -> Vec<i64> {
    let mut rng = rand::rng();
    (0..n).map(|_| rng.random_range(0..1_000_000_000i64)).collect()
}

// annenhvert element likt
fn make_many_duplicates(random_data: &[i64]) -> Vec<i64> {
    let mut arr = random_data.to_vec();
    let mut i = 1;
    while i < arr.len() {
        arr[i] = arr[i - 1];
        i += 2;
    }
    arr
}

fn make_sorted(random_data: &[i64]) -> Vec<i64> {
    let mut arr = random_data.to_vec();
    arr.sort();
    arr
}

fn make_reversed(sorted_data: &[i64]) -> Vec<i64> {
    let mut arr = sorted_data.to_vec();
    arr.reverse();
    arr
}

// tidsmåling
fn time_sort(name: &str, dataset_name: &str, data: &[i64], sort_fn: fn(&mut [i64])) -> u128 {
    let mut arr = data.to_vec();
    let checksum_before = checksum(&arr);

    let start = Instant::now();
    sort_fn(&mut arr);
    let elapsed = start.elapsed();

    let checksum_after = checksum(&arr);
    let sorted_ok = is_sorted(&arr);
    let checksum_ok = checksum_before == checksum_after;

    if !checksum_ok || !sorted_ok {
        println!(
            "FEIL! {} på \"{}\": sjekksum_ok={} sortert_ok={}",
            name, dataset_name, checksum_ok, sorted_ok
        );
        assert!(checksum_ok, "Sjekksumtest feilet for {} på {}", name, dataset_name);
        assert!(sorted_ok, "Rekkefølgetest feilet for {} på {}", name, dataset_name);
    }

    println!(
        "{:<12} {:<20} {:>15} ms",
        name,
        dataset_name,
        elapsed.as_millis()
    );

    elapsed.as_millis()
}

fn main() {
    let mut small: Vec<i64> = vec![5, -3, 9, 9, 0, 2, -3, 100, 1, 1, 1, 42];
    let expected_sum: i64 = small.iter().sum();
    let mut small_dual = small.clone();
    quicksort_single(&mut small);
    quicksort_dual(&mut small_dual);
    assert!(is_sorted(&small), "quicksort_single sorterer feil på lite datasett");
    assert!(is_sorted(&small_dual), "quicksort_dual sorterer feil på lite datasett");
    assert_eq!(checksum(&small), expected_sum, "quicksort_single mistet data på lite datasett");
    assert_eq!(checksum(&small_dual), expected_sum, "quicksort_dual mistet data på lite datasett");
    assert_eq!(small, small_dual, "de to sorteringene gir ulikt resultat på lite datasett");
    println!("Sanity-sjekk på lite datasett: OK\n");

    let n: usize = 50_000_000;
    println!("Sorterer tabeller med n = {} tall\n", n);

    let random_data = generate_random(n);
    let duplicates_data = make_many_duplicates(&random_data);
    let sorted_data = make_sorted(&random_data);
    let reversed_data = make_reversed(&sorted_data);

    let datasets: Vec<(&str, &Vec<i64>)> = vec![
        ("tilfeldige", &random_data),
        ("mange duplikater", &duplicates_data),
        ("sortert", &sorted_data),
        ("baklengs sortert", &reversed_data),
    ];

    println!("{:<12} {:<20} {:>18}", "Algoritme", "Datasett", "Tid");
    println!("{}", "-".repeat(55));

    let mut results: Vec<(&str, &str, u128)> = vec![];

    for (dataset_name, data) in &datasets {
        let t_single = time_sort("single-pivot", dataset_name, data, quicksort_single);
        results.push(("single-pivot", dataset_name, t_single));

        let t_dual = time_sort("dual-pivot", dataset_name, data, quicksort_dual);
        results.push(("dual-pivot", dataset_name, t_dual));
    }

    println!("\nOppsummering: raskest algoritme per datasett");
    println!("{}", "-".repeat(55));
    for (dataset_name, _) in &datasets {
        let single_t = results
            .iter()
            .find(|(alg, d, _)| *alg == "single-pivot" && d == dataset_name)
            .unwrap()
            .2;
        let dual_t = results
            .iter()
            .find(|(alg, d, _)| *alg == "dual-pivot" && d == dataset_name)
            .unwrap()
            .2;
        let winner = if single_t < dual_t {
            "single-pivot"
        } else if dual_t < single_t {
            "dual-pivot"
        } else {
            "uavgjort"
        };
        println!(
            "{:<20} single-pivot={:>7} ms  dual-pivot={:>7} ms  -> raskest: {}",
            dataset_name, single_t, dual_t, winner
        );
    }
}
