## Travel to work area algorithm in Rust

This crate aims to replicate the travel to work area algorithm used by the ONS to identifying labour market areas in the UK using commute matrices from the census.
The methodology is explained in detail [here](https://www.ncl.ac.uk/media/wwwnclacuk/curds/files/TTWA%20report.pdf).
This is for the most part a carbon copy of the methodology, except the document never outlines exactly what the 'X' equation is.
Consequently, I've assumed that the score is 1.0 if either the population is above the minimum (3,500) and the self-containment is above the target (0.75) or the population is above the target (25,000) and the self-containment is above the minimum (0.667).
If it fails both of these tests, the the score is calculated such that it will be equal to 0.0 if the population is 3,500 and the self-containment is 0.75 and if the population is 25,000 and the self-containment is 0.667, with a linear trade-off in between.

## Usage

The code takes a CSV where rows represent origin locations and columns represent destination locations. The value at row i, column j is the number of people who live in area i and community to area j.
You need to have Rust and cargo installed to run it, but you can do so simply by typing

```bash
cargo run --release path/to/your/file.csv path/to/result/file.csv
```

This will create a new CSV with the specified file name where each row contains a location and the TTWA it belongs to. The TTWA numbers are based on the initial TTWA assignment and don't mean anything in and of themselves.

The ONS has provided travel to work matrices based on the 2021 England and Wales Census that can be used with this program [here](https://www.ons.gov.uk/releases/estimationoftraveltoworkmatricesenglandandwales).


## Future plans

I hope to gain access to another paper that appears to outline what the actual parameterization of the 'X' equation is, at which point I will update the algorithm as necessary.
I'll also add some quality of life improvements such as making it so others can use the program without installing Rust/cargo (I'm very new with Rust development so bear with me!).