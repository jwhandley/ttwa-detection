## Travel to work area algorithm in Rust

This crate aims to replicate the travel to work area algorithm used by the ONS to identifying labour market areas in the UK using commute matrices from the census.
The methodology is explained in detail [here](https://www.ncl.ac.uk/media/wwwnclacuk/curds/files/TTWA%20report.pdf).
This is for the most part a carbon copy of the methodology, except the document never outlines exactly what the 'X' equation is.
Consequently, I've assumed that the score for an area is a linear combination of its (resident) population and its self-containment, rescaled so that they are both equal to 0 at the minimum allowed value (3,500 and 2/3, respectively) and both equal to 1 at the target value (25,000 and 0.75, respectively).

## Usage

At the moment the code takes a CSV where rows represent origin locations and columns represent destination locations. The value at row i, column j is the number of people who live in area i and community to area j.
You need to have Rust and cargo installed to run it, but you can do so simply by typing

```bash
cargo run --release path/to/your/file.csv
```

The ONS has provided travel to work matrices based on the 2021 England and Wales Census that can be used with this program [here](https://www.ons.gov.uk/releases/estimationoftraveltoworkmatricesenglandandwales).


## Future plans

I hope to gain access to another paper that appears to outline what the actual parameterization of the 'X' equation is, at which point I will update the algorithm as necessary.
I'll also add some quality of life improvements such as saving the results to a CSV and possibly making it so others can use the program without installing Rust/cargo (I'm very new with Rust development so bear with me!).