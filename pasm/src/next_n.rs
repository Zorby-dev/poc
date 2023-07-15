pub trait NextN<T>: Iterator<Item = T> {
    fn next_n<B: FromIterator<T>>(&mut self, n: usize) -> B {
        let mut out = vec![];

        for _ in 0..n {
            match self.next() {
                Some(next) => out.push(next),
                None => break
            }
        }

        out.into_iter().collect()
    }
}

impl<T, I: Iterator<Item = T> + Sized> NextN<T> for I {}