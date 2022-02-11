
object OurStruct {
    fn print();
}

fn create_s(x: u32, y: u32) -> OurStruct;

object OurStructList {
    fn add(s: OurStruct);
    fn print();
    fn get(index: usize) -> Option<OurStruct>;
}

fn new_struct_list() -> OurStructList;

fn create_ss() -> OurStructList;

fn print_ss(ss: OurStructList);