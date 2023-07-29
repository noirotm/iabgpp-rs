use num_traits::ops::checked::CheckedAdd;
use num_traits::One;

/// Iterative generic fibonacci implementation
pub struct Fibonacci<T> {
    curr: Option<T>,
    next: Option<T>,
}

impl<T> Iterator for Fibonacci<T>
where
    T: CheckedAdd + Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let next = self.curr?.checked_add(&self.next?);

        self.curr = self.next;
        self.next = next;

        self.curr
    }
}

/// Create a new Iterative fibonacci.
pub fn fibonacci_iterator<T>() -> Fibonacci<T>
where
    T: One + Copy,
{
    let init = T::one();
    Fibonacci {
        curr: Some(init),
        next: Some(init),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_is_correct() {
        assert_eq!(
            fibonacci_iterator::<u16>().take(16).collect::<Vec<_>>(),
            vec![1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610, 987, 1597]
        );
    }

    #[test]
    fn u8_overflow() {
        assert_eq!(
            fibonacci_iterator::<u8>().collect::<Vec<_>>(),
            vec![1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233]
        );
    }
}
