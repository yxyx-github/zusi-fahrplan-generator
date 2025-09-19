# zusi-fahrplan-generator

CLI-tool to generate timetables for [Zusi 3](https://www.zusi.de/) (consisting of `.fpn`, `.trn` and `.timetable.xml` files).

For local build the following dependencies need to be cloned separately:
* [zusi-xml-lib](https://github.com/yxyx-github/rust-zusi-xml-lib)
* [serde-helpers](https://github.com/yxyx-github/rust-serde-helpers)

By now unix file paths are used in the tests because development is done on Linux. Therefor the tests might fail on Windows.