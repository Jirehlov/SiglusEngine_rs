const IMAGE_EXT_CANDIDATES: [&str; 5] = ["g00", "bmp", "png", "jpg", "dds"];

trait PropIntExt {
    fn as_int(&self) -> Option<i32>;
}

impl PropIntExt for siglus::vm::Prop {
    fn as_int(&self) -> Option<i32> {
        if let siglus::vm::PropValue::Int(v) = self.value {
            Some(v)
        } else {
            None
        }
    }
}
