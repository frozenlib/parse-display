use parse_display::*;
use std::fmt::Display;

    #[derive(Display)]
    enum TestEnum {
        #[display("{} = {x}")]
        A {
            #[display("---{l}")]
            x: TestStruct,
        },
    }

    struct TestStruct {
        l: String,
    }

