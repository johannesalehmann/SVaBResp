pub struct Table {
    headers: Vec<TableHeader>,
    rows: Vec<TableRow>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn get_width(&self) -> usize {
        self.headers
            .first()
            .expect("Can only compute table width if it has at least one header")
            .get_width()
    }

    pub fn start_new_header(&mut self) {
        self.headers.push(TableHeader::new());
    }

    pub fn add_to_header<S: Into<String>>(&mut self, name: S, width: usize) {
        let index = self.headers.len() - 1;
        self.headers[index]
            .entries
            .push(TableHeaderEntry::new(name, width));
    }

    pub fn start_new_row<S: Into<String>>(&mut self, name: S) {
        self.rows.push(TableRow::new(name));
    }

    pub fn add_timeout(&mut self) {
        let index = self.rows.len() - 1;
        self.rows[index].entries.push(TableTime::Timeout)
    }
    pub fn add_runtime(&mut self, seconds: f64) {
        let index = self.rows.len() - 1;
        self.rows[index].entries.push(TableTime::Seconds(seconds))
    }

    pub fn print_latex(&self) {
        let width = self.get_width();
        println!("\\begin{{tabular}}{{l{}}}", "r".repeat(width - 1));
        println!("    \\toprule");
        for (index, header) in self.headers.iter().enumerate() {
            assert_eq!(width, header.get_width());
            for entry in &header.entries {
                println!(
                    "    & \\multicolumn{{{}}}{{c}}{{\\emph{{{}}}}}",
                    entry.width, entry.name
                )
            }
            print!("    \\\\");
            let mut location = 1;
            if index + 1 != self.headers.len() {
                print!("    ");
                for entry in &header.entries {
                    if !entry.name.is_empty() {
                        print!("\\cmidrule(lr){{{}-{}}}", location, location + entry.width);
                    }

                    location += entry.width;
                }
                println!();
            }
        }
        println!("\\midrule");

        for (_row_index, row) in self.rows.iter().enumerate() {
            println!("    \\texttt{{{}}} &", row.name);
            for (entry_index, entry) in row.entries.iter().enumerate() {
                match entry {
                    TableTime::Timeout => {
                        print!("        & TO")
                    }
                    TableTime::Seconds(s) => {
                        if *s < 1.0 {
                            print!("        & \\statCell[ms]{{{:.0}}}", s * 1000.0)
                        } else {
                            print!("        & \\statCell{{{:.2}}}", s)
                        }
                    }
                }
                if entry_index < row.entries.len() - 1 {
                    println!(" &");
                } else {
                    println!(" \\\\");
                }
            }
        }
        println!("    \\bottomrule");
        println!("\\end{{tabular}}");
    }
}

pub struct TableRow {
    name: String,
    entries: Vec<TableTime>,
}

impl TableRow {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            entries: Vec::new(),
        }
    }
}

pub enum TableTime {
    Timeout,
    Seconds(f64),
}

pub struct TableHeader {
    entries: Vec<TableHeaderEntry>,
}

impl TableHeader {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get_width(&self) -> usize {
        self.entries.iter().map(|e| e.width).sum::<usize>() + 1
    }
}

pub struct TableHeaderEntry {
    name: String,
    width: usize,
}

impl TableHeaderEntry {
    pub fn new<S: Into<String>>(name: S, width: usize) -> Self {
        Self {
            name: name.into(),
            width,
        }
    }
}
