fn process(items: &[u32]) -> u32 {
    let mut sum = 0;
    for x in items {
        if x % 2 == 0 {
            sum += x * 2;
        }
    }
    sum
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];
    let result = process(&numbers);
    println!("The result is: {result}");
}
