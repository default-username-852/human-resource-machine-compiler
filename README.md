# Human Resource Machine Compiler
A compiler for the game "Human resource machine"

## Installation
Clone this repo and build the project with cargo.

## Usage
Make a file called in.txt and put your code you want to compile in it. Then, run the program. It should create a file called out.txt. Copy the contents of that file and hit the paste button inside Human resource machine.

### Syntax
The expressions that the compiler accepts are the following:

#### IO
- `input()`
- `output(a)`

#### Mathematical
- `a + b`
- `a - b`

When adding and subtracting, the compiler uses tile 0 as an intermediate tile to store numbers. For some problems though, important data is stored in tile 0. To change which tile is used as an intermediate, add `#add_square number` where number is the tile to use as a temporary tile.

#### Logical
- `a > b`
- `a < b`
- `a == b`
- `a != b`
- `a >= b`
- `a <= b`

#### Assignment
- `a = b`
 Evaluates b and stores it in a

#### Flow control
- `if (a) {b;}`
- `loop {a;}`
- `while (a) {b;}`

#### Numbers
A square is referenced by typing `*{tile number}` so for example, to reference a value in tile 3, you type `*3`.

To reference the square that a number is pointing to, you type `**{tile number}`. For example, if you wanted to reference the value that the number in tile 4 is pointing at, you type `**4`.

#### Macros
The compiler does not support variable names. It does though support macros. To define a macro, add `#define from to` somewhere in `in.txt`. The compiler will then replace every occurence of `from` with `to`. You can then emulate variables by adding `#define variable *5`.

For examples, see the examples folder.


--------

The compiler is pretty untested and will probably crash or get stuck if it encounters something it doesn't like. It has been verified to be able to compile code that beats most of the challenges though.

## License
[MIT](https://choosealicense.com/licenses/mit/)
