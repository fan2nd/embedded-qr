use embedded_qr::{
    QrBuilder, QrMatrix, Version, Version1, Version2, Version3, Version4, Version5, Version6,
    Version7, Version8, Version9, Version10, Version11, Version12, Version13, Version14, Version15,
    Version16, Version17, Version18, Version19, Version20, Version21, Version22, Version23,
    Version24, Version25, Version26, Version27, Version28, Version29, Version30, Version31,
    Version32, Version33, Version34, Version35, Version36, Version37, Version38, Version39,
    Version40,
};

const QUIET_ZONE: usize = 2;
const DARK: &str = "\x1b[40m  ";
const LIGHT: &str = "\x1b[47m  ";
const RESET: &str = "\x1b[0m";

fn main() {
    render_version::<Version1>("Version1", 1);
    render_version::<Version2>("Version2", 2);
    render_version::<Version3>("Version3", 3);
    render_version::<Version4>("Version4", 4);
    render_version::<Version5>("Version5", 5);
    render_version::<Version6>("Version6", 6);
    render_version::<Version7>("Version7", 7);
    render_version::<Version8>("Version8", 8);
    render_version::<Version9>("Version9", 9);
    render_version::<Version10>("Version10", 10);
    render_version::<Version11>("Version11", 11);
    render_version::<Version12>("Version12", 12);
    render_version::<Version13>("Version13", 13);
    render_version::<Version14>("Version14", 14);
    render_version::<Version15>("Version15", 15);
    render_version::<Version16>("Version16", 16);
    render_version::<Version17>("Version17", 17);
    render_version::<Version18>("Version18", 18);
    render_version::<Version19>("Version19", 19);
    render_version::<Version20>("Version20", 20);
    render_version::<Version21>("Version21", 21);
    render_version::<Version22>("Version22", 22);
    render_version::<Version23>("Version23", 23);
    render_version::<Version24>("Version24", 24);
    render_version::<Version25>("Version25", 25);
    render_version::<Version26>("Version26", 26);
    render_version::<Version27>("Version27", 27);
    render_version::<Version28>("Version28", 28);
    render_version::<Version29>("Version29", 29);
    render_version::<Version30>("Version30", 30);
    render_version::<Version31>("Version31", 31);
    render_version::<Version32>("Version32", 32);
    render_version::<Version33>("Version33", 33);
    render_version::<Version34>("Version34", 34);
    render_version::<Version35>("Version35", 35);
    render_version::<Version36>("Version36", 36);
    render_version::<Version37>("Version37", 37);
    render_version::<Version38>("Version38", 38);
    render_version::<Version39>("Version39", 39);
    render_version::<Version40>("Version40", 40);
}

fn render_version<T: Version>(label: &str, version_number: usize) {
    let data = format!("hello {version_number}");
    let matrix = match QrBuilder::<T>::new().build(data.as_bytes()) {
        Ok(matrix) => matrix,
        Err(error) => {
            println!("{label}: failed to encode {data:?}: {error:?}");
            println!();
            return;
        }
    };

    println!(
        "{label}  width={}  ecc={:?}  mask={:?}",
        matrix.width(),
        matrix.ecc_level(),
        matrix.mask()
    );
    println!("data={data:?}");
    print_matrix(&matrix);
    println!();
}

fn print_matrix<T: Version>(matrix: &QrMatrix<T>) {
    let quiet_row = LIGHT.repeat(matrix.width() + QUIET_ZONE * 2);
    for _ in 0..QUIET_ZONE {
        println!("{quiet_row}{RESET}");
    }

    for y in 0..matrix.width() {
        print!("{}", LIGHT.repeat(QUIET_ZONE));
        for x in 0..matrix.width() {
            print!("{}", if matrix.get(x, y) { DARK } else { LIGHT });
        }
        print!("{}", LIGHT.repeat(QUIET_ZONE));
        println!("{RESET}");
    }

    for _ in 0..QUIET_ZONE {
        println!("{quiet_row}{RESET}");
    }
}
