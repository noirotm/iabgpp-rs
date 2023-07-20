/// Iterative fibonacci implementation
pub struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let next = self.curr + self.next;

        self.curr = self.next;
        self.next = next;

        Some(self.curr)
    }
}

/// Create a new Iterative fibonacci.
pub fn fibonacci_iterator() -> Fibonacci {
    Fibonacci { curr: 1, next: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci() {
        assert_eq!(
            fibonacci_iterator().take(12).collect::<Vec<_>>(),
            vec![1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233]
        );
    }
}
