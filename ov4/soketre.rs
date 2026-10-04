use std::env;
use std::io::{self, Read};

struct Node {
    word: String,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(word: String) -> Self {
        Node { word, left: None, right: None }
    }

    fn insert(&mut self, word: String) {
        if word == self.word {
            return; // ordet finnes allerede, ikke legg inn dupliker
        }
        let child = if word < self.word { &mut self.left } else { &mut self.right };
        match child {
            Some(node) => node.insert(word),
            None => *child = Some(Box::new(Node::new(word))),
        }
    }
}

struct SearchTree {
    root: Option<Box<Node>>,
}

impl SearchTree {
    fn new() -> Self {
        SearchTree { root: None }
    }

    fn insert(&mut self, word: String) {
        match &mut self.root {
            Some(node) => node.insert(word),
            None => self.root = Some(Box::new(Node::new(word))),
        }
    }

    // Samler ordene på et gitt nivå (roten er nivå 0), venstre mot høyre.
    // Manglende noder gir `None`, slik at posisjonen fortsatt tar opp plass.
    fn collect_level(node: Option<&Node>, level: usize, out: &mut Vec<Option<String>>) {
        if level == 0 {
            out.push(node.map(|n| n.word.clone()));
            return;
        }
        let (left, right) = match node {
            Some(n) => (n.left.as_deref(), n.right.as_deref()),
            None => (None, None),
        };
        Self::collect_level(left, level - 1, out);
        Self::collect_level(right, level - 1, out);
    }

    // Skriver ut de fire øverste nivåene. Roten får 64 tegns plass, og
    // bredden halveres for hvert nivå nedover, slik oppgaveteksten foreslår.
    fn print_levels(&self, num_levels: usize, total_width: usize) {
        let root = self.root.as_deref();
        for level in 0..num_levels {
            let width = total_width / (1 << level);
            if width == 0 {
                break;
            }
            let mut words = Vec::new();
            Self::collect_level(root, level, &mut words);

            let line: String = words
                .into_iter()
                .map(|w| format!("{:^width$}", w.unwrap_or_default(), width = width))
                .collect();
            println!("{}", line.trim_end());
        }
    }
}

fn read_words_from_stdin() -> Vec<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("klarte ikke å lese fra stdin");
    input.split_whitespace().map(|s| s.to_string()).collect()
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let words = if !args.is_empty() {
        args
    } else {
        println!("Skriv inn ord (et vilkårlig antall, avslutt med Ctrl+D / Ctrl+Z):");
        read_words_from_stdin()
    };

    let mut tree = SearchTree::new();
    for word in words {
        tree.insert(word);
    }

    tree.print_levels(4, 64);
}
