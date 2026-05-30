// esp-idf build glue. Required by esp-idf-sys to locate/build the IDF and
// embed sdkconfig + partition table. License: PolyForm Noncommercial 1.0.0
fn main() {
    embuild::espidf::sysenv::output();
}
