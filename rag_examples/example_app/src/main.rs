fn process(items: &[u32]) -> u32 {
    helper(items)
}

fn helper(items: &[u32]) -> u32 {
    items.iter().filter(|&&x| x % 2 == 0).map(|&x| x * 2).sum()
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let result = process(&numbers);
    println!("The result is: {result}");
}
