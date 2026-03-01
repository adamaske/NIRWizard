/// Standalone HDF5 tree inspector.
/// Usage:  cargo run --bin inspect_snirf -- <path-to-file.snirf>
use hdf5::File;
use std::ops::Deref;

fn dtype_label(ds: &hdf5::Dataset) -> String {
    use hdf5::types::TypeDescriptor;
    ds.dtype()
        .ok()
        .and_then(|dt| dt.to_descriptor().ok())
        .map(|desc| match desc {
            TypeDescriptor::Float(_) => "f64".into(),
            TypeDescriptor::Integer(_) => "i32".into(),
            TypeDescriptor::Unsigned(_) => "u32".into(),
            TypeDescriptor::Boolean => "bool".into(),
            TypeDescriptor::VarLenUnicode => "str".into(),
            TypeDescriptor::VarLenAscii => "ascii".into(),
            TypeDescriptor::FixedAscii(n) => format!("ascii[{}]", n),
            TypeDescriptor::FixedUnicode(n) => format!("str[{}]", n),
            _ => format!("{:?}", desc),
        })
        .unwrap_or_else(|| "?".into())
}

fn scalar_preview(ds: &hdf5::Dataset) -> String {
    let n: usize = ds.shape().iter().product();
    let is_scalar = ds.ndim() == 0 || (ds.ndim() == 1 && n == 1);
    if !is_scalar {
        return String::new();
    }
    if let Ok(s) = ds.read_scalar::<hdf5::types::VarLenUnicode>() {
        return format!(" = {:?}", s.to_string());
    }
    if let Ok(v) = ds.read_scalar::<f64>() {
        return format!(" = {}", v);
    }
    if let Ok(v) = ds.read_scalar::<i32>() {
        return format!(" = {}", v);
    }
    String::new()
}

fn walk(group: &hdf5::Group, depth: usize) {
    let indent = "  ".repeat(depth);
    let names = match group.member_names() {
        Ok(n) => n,
        Err(e) => {
            println!("{}[err listing: {}]", indent, e);
            return;
        }
    };
    for name in &names {
        if let Ok(ds) = group.dataset(name) {
            let shape = ds.shape();
            let ty = dtype_label(&ds);
            let preview = scalar_preview(&ds);
            println!("{}[D] {}  {:?} <{}>{}",  indent, name, shape, ty, preview);
        } else if let Ok(sub) = group.group(name) {
            println!("{}[G] {}/", indent, name);
            walk(&sub, depth + 1);
        } else {
            println!("{}[?] {}", indent, name);
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("Usage: inspect_snirf <path-to-file.snirf>");
    let file = File::open(&path).unwrap_or_else(|e| panic!("Failed to open file: {}", e));
    println!("\n=== HDF5 tree: {} ===\n", path);
    walk(file.deref(), 0);
    println!("\n=== end ===");
}
