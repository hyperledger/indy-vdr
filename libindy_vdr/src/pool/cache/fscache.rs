use sled;

pub fn test() {
    let sled = sled::open("my_db").unwrap();
    sled.insert(b"yo!", vec![1, 2, 3]).unwrap();
}
