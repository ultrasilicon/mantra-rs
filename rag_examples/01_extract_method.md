Before:
fn process(items: &[u32]) -> u32 {
    let mut sum = 0;
    for x in items {
        if x % 2 == 0 {
            sum += x * 2;
        }
    }
    sum
}

After (pattern):
fn process(items: &[u32]) -> u32 {
    sum_even_double(items)
}

fn sum_even_double(items: &[u32]) -> u32 {
    items.iter().filter(|x| *(*x) % 2 == 0).map(|x| x * 2).sum()
}
