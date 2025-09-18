use term_cursor as cursor;
fn main() {
    // Clear screen once
    print!("{}", cursor::Clear);
    // Print static labels
    print!("{}CPU: ", cursor::Goto(0, 1));
    print!("{}Memory: ", cursor::Goto(0, 2));

    // Now update just the values in a loop
    for i in 0..10 {
        // Go to where the CPU value should be (after "CPU: ")
        print!("{}{}%", cursor::Goto(5, 1), i * 10);
        // Go to where the Memory value should be
        print!("{}{}GB", cursor::Goto(8, 2), i);

        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}
