use std::collections::HashMap;

use lmntalc::util::Span;

use crate::symbol::Symbol;

#[derive(Debug)]
pub struct RefereceMap {
    symbol_seq: Vec<Symbol>,
    references: HashMap<usize, Vec<usize>>,
}

impl RefereceMap {
    pub fn new(refs: Vec<Vec<Span>>, normal_symbol: Vec<Span>) -> Self {
        let mut symbol_seq = Vec::new();
        let mut references = HashMap::new();

        for group in &refs {
            for span in group {
                let symbol = Symbol::new(*span);
                symbol_seq.push(symbol);
            }
        }

        for span in &normal_symbol {
            let symbol = Symbol::new(*span);
            symbol_seq.push(symbol);
        }

        symbol_seq.sort();
        symbol_seq.dedup();
        // map every symbol to its index
        let map = symbol_seq
            .iter()
            .enumerate()
            .map(|(i, symbol)| (*symbol, i))
            .collect::<HashMap<_, _>>();

        for group in &refs {
            // insert theme alternatevely
            // [1, 2, 3] => [1, [2, 3]], [2, [1, 3]], [3, [1, 2]]
            for (i, span) in group.iter().enumerate() {
                let symbol = Symbol::new(*span);
                let index = *map.get(&symbol).unwrap();
                let refs = references.entry(index).or_insert_with(Vec::new);
                for (j, other_span) in group.iter().enumerate() {
                    if i != j {
                        let other_symbol = Symbol::new(*other_span);
                        let other_index = *map.get(&other_symbol).unwrap();
                        refs.push(other_index);
                    }
                }
            }
        }

        Self {
            symbol_seq,
            references,
        }
    }

    pub fn query(&self, line: u32, col: u32) -> Option<Symbol> {
        find(line, col, &self.symbol_seq).map(|i| self.symbol_seq[i])
    }

    pub fn query_references(&self, line: u32, col: u32) -> Option<Vec<Symbol>> {
        let index = find(line, col, &self.symbol_seq)?;
        let refs = self.references.get(&index)?;
        Some(refs.iter().map(|&i| self.symbol_seq[i]).collect())
    }

    pub fn query_references_with_self(&self, line: u32, col: u32) -> Option<Vec<Symbol>> {
        let index = find(line, col, &self.symbol_seq)?;
        if let Some(refs) = self.references.get(&index) {
            let mut result = refs.iter().map(|&i| self.symbol_seq[i]).collect::<Vec<_>>();
            result.push(self.symbol_seq[index]);
            Some(result)
        } else {
            Some(vec![self.symbol_seq[index]])
        }
    }
}

/// Find if there is a symbol at the given position, and return the index of the symbol in the symbol sequence.
fn find(line: u32, col: u32, refs: &Vec<Symbol>) -> Option<usize> {
    if refs.is_empty() {
        return None;
    }
    let mut low = 0;
    let mut high = refs.len() - 1;
    while low <= high {
        let mid = (low + high) / 2;
        let mid_val = refs[mid];
        if mid_val.is_inside(line, col) {
            return Some(mid);
        }
        if mid_val.line < line || (mid_val.line == line && mid_val.col < col) {
            low = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        }
    }
    None
}

#[test]
fn test_find() {
    let refs = vec![
        Symbol {
            line: 0,
            col: 0,
            length: 1,
        },
        Symbol {
            line: 0,
            col: 2,
            length: 1,
        },
        Symbol {
            line: 1,
            col: 1,
            length: 2,
        },
        Symbol {
            line: 1,
            col: 4,
            length: 1,
        },
    ];
    assert_eq!(find(0, 0, &refs), Some(0));
    assert_eq!(find(0, 1, &refs), Some(0));

    assert_eq!(find(0, 2, &refs), Some(1));
    assert_eq!(find(0, 3, &refs), Some(1));

    assert_eq!(find(0, 4, &refs), None);

    assert_eq!(find(1, 0, &refs), None);
    assert_eq!(find(1, 1, &refs), Some(2));
    assert_eq!(find(1, 2, &refs), Some(2));
    assert_eq!(find(1, 3, &refs), Some(2));

    assert_eq!(find(1, 4, &refs), Some(3));
    assert_eq!(find(1, 5, &refs), Some(3));
}
