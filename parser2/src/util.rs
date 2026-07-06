use std::collections::HashMap;
use std::hash::Hash;

pub(crate) fn find_duplicates<I, KFN, K, DFN>(iter: I, mut key_fn: KFN, mut dup_fn: DFN)
where
    I: IntoIterator,
    KFN: FnMut(&I::Item) -> K,
    K: Hash + Eq,
    DFN: FnMut(I::Item, &I::Item),
{
    let mut candidates: HashMap<_, Vec<_>> = HashMap::new();

    for elem in iter {
        candidates.entry(key_fn(&elem)).or_default().push(elem);
    }

    for (_, candidates) in candidates {
        if candidates.len() <= 1 {
            continue;
        }

        let mut duplicates = candidates.into_iter();
        let first = duplicates.next().unwrap();

        for duplicate in duplicates {
            dup_fn(duplicate, &first);
        }
    }
}
