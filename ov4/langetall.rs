use std::cell::RefCell;
use std::cmp::Ordering;
use std::env;
use std::io::{self, Write};
use std::rc::{Rc, Weak};

struct Node {
    digit: u8,
    next: Option<Rc<RefCell<Node>>>,
    prev: Option<Weak<RefCell<Node>>>,
}

// `head` er det mest signifikante sifferet, `tail` det minst signifikante
// (som å lese tallet fra venstre mot høyre).
struct DigitList {
    head: Option<Rc<RefCell<Node>>>,
    tail: Option<Rc<RefCell<Node>>>,
    len: usize,
}

impl DigitList {
    fn new() -> Self {
        DigitList { head: None, tail: None, len: 0 }
    }

    fn push_back(&mut self, digit: u8) {
        let node = Rc::new(RefCell::new(Node { digit, next: None, prev: None }));
        match self.tail.take() {
            Some(old_tail) => {
                old_tail.borrow_mut().next = Some(Rc::clone(&node));
                node.borrow_mut().prev = Some(Rc::downgrade(&old_tail));
                self.tail = Some(node);
            }
            None => {
                self.head = Some(Rc::clone(&node));
                self.tail = Some(node);
            }
        }
        self.len += 1;
    }

    fn push_front(&mut self, digit: u8) {
        let node = Rc::new(RefCell::new(Node { digit, next: None, prev: None }));
        match self.head.take() {
            Some(old_head) => {
                node.borrow_mut().next = Some(Rc::clone(&old_head));
                old_head.borrow_mut().prev = Some(Rc::downgrade(&node));
                self.head = Some(node);
            }
            None => {
                self.head = Some(Rc::clone(&node));
                self.tail = Some(node);
            }
        }
        self.len += 1;
    }

    fn from_digits(s: &str) -> Self {
        let mut list = DigitList::new();
        for c in s.chars() {
            let d = c.to_digit(10).expect("tallet inneholder et ugyldig siffer");
            list.push_back(d as u8);
        }
        list
    }

    fn to_digit_string(&self) -> String {
        let mut s = String::with_capacity(self.len);
        let mut current = self.head.clone();
        while let Some(node) = current {
            s.push(char::from_digit(node.borrow().digit as u32, 10).unwrap());
            current = node.borrow().next.clone();
        }
        s
    }

    // Fjerner ledende nuller (beholder minst ett siffer).
    fn strip_leading_zeros(&mut self) {
        while self.len > 1 {
            let is_zero = self.head.as_ref().map(|n| n.borrow().digit == 0).unwrap_or(false);
            if !is_zero {
                break;
            }
            let old_head = self.head.take().unwrap();
            let new_head = old_head.borrow().next.clone();
            match &new_head {
                Some(nh) => nh.borrow_mut().prev = None,
                None => self.tail = None,
            }
            self.head = new_head;
            self.len -= 1;
        }
    }
}

fn prev_of(node: &Rc<RefCell<Node>>) -> Option<Rc<RefCell<Node>>> {
    node.borrow().prev.as_ref().and_then(|w| w.upgrade())
}

// Sammenligner størrelsen på to (ikke-negative) tall representert som sifferlister.
fn compare(a: &DigitList, b: &DigitList) -> Ordering {
    if a.len != b.len {
        return a.len.cmp(&b.len);
    }
    let mut na = a.head.clone();
    let mut nb = b.head.clone();
    while na.is_some() && nb.is_some() {
        let x = na.clone().unwrap();
        let y = nb.clone().unwrap();
        let (dx, dy) = (x.borrow().digit, y.borrow().digit);
        if dx != dy {
            return dx.cmp(&dy);
        }
        na = x.borrow().next.clone();
        nb = y.borrow().next.clone();
    }
    Ordering::Equal
}

// Går gjennom begge listene fra bakerst (minst signifikant) og mot forsiden
// via prev-pekerne, og legger sammen ett og ett siffer om gangen.
fn add_magnitudes(a: &DigitList, b: &DigitList) -> DigitList {
    let mut result = DigitList::new();
    let mut na = a.tail.clone();
    let mut nb = b.tail.clone();
    let mut carry: u8 = 0;

    while na.is_some() || nb.is_some() || carry > 0 {
        let da = na.as_ref().map(|n| n.borrow().digit).unwrap_or(0);
        let db = nb.as_ref().map(|n| n.borrow().digit).unwrap_or(0);
        let sum = da + db + carry;
        result.push_front(sum % 10);
        carry = sum / 10;
        na = na.and_then(|n| prev_of(&n));
        nb = nb.and_then(|n| prev_of(&n));
    }
    result
}

// Forutsetter at `larger` >= `smaller`. Går gjennom begge listene fra
// bakerst og mot forsiden via prev-pekerne, og låner fra neste siffer ved behov.
fn sub_magnitudes(larger: &DigitList, smaller: &DigitList) -> DigitList {
    let mut result = DigitList::new();
    let mut na = larger.tail.clone();
    let mut nb = smaller.tail.clone();
    let mut borrow: i8 = 0;

    while na.is_some() {
        let da = na.as_ref().map(|n| n.borrow().digit as i8).unwrap_or(0);
        let db = nb.as_ref().map(|n| n.borrow().digit as i8).unwrap_or(0);
        let mut diff = da - db - borrow;
        if diff < 0 {
            diff += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        result.push_front(diff as u8);
        na = na.and_then(|n| prev_of(&n));
        nb = nb.and_then(|n| prev_of(&n));
    }
    result.strip_leading_zeros();
    result
}

fn read_interactive() -> (String, String, String) {
    let mut num1 = String::new();
    let mut op = String::new();
    let mut num2 = String::new();

    print!("Skriv inn første tall: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut num1).unwrap();

    print!("Skriv inn + eller -: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut op).unwrap();

    print!("Skriv inn andre tall: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut num2).unwrap();

    (num1.trim().to_string(), op.trim().to_string(), num2.trim().to_string())
}

fn print_regnestykke(a: &str, op: char, b: &str, result: &str) {
    let width = a.len().max(b.len()).max(result.len());
    println!("{:>width$}", a, width = width + 2);
    println!("{} {:>width$}", op, b, width = width);
    println!("= {:>width$}", result, width = width);
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let (num1_str, op_str, num2_str) = if args.len() >= 3 {
        (args[0].clone(), args[1].clone(), args[2].clone())
    } else {
        read_interactive()
    };

    let op = op_str.trim().chars().next().expect("mangler operator (+ eller -)");
    let a = DigitList::from_digits(num1_str.trim());
    let b = DigitList::from_digits(num2_str.trim());

    let (result, negative) = match op {
        '+' => (add_magnitudes(&a, &b), false),
        '-' => match compare(&a, &b) {
            Ordering::Less => (sub_magnitudes(&b, &a), true),
            _ => (sub_magnitudes(&a, &b), false),
        },
        _ => panic!("ukjent operator '{}', bruk + eller -", op),
    };

    let mut result_str = result.to_digit_string();
    if negative && result_str != "0" {
        result_str = format!("-{}", result_str);
    }

    print_regnestykke(num1_str.trim(), op, num2_str.trim(), &result_str);
}
