The Token standard is in JCT01/src/main.rs
The script for local tests is in tests/src/main.rs

To run the test script:
cd tests
./build.sh
./ptest.sh        # when using println in test script (my current default)
cargo test        # when using assert! in test script
