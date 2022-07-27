# rust-lang

## Project2

下面会对所使用到的文档都做一下记录（简单翻译）

### Part1 错误处理

[如何使用 Failure](https://boats.gitlab.io/failure/fail.html)


##### `Fail` 特征

`Fail` 特征 作为标准库中 `std::error::Error` 的代替，需要设计成为支持以下一系列的操作：
- `Fail` 特征被 `Debug` 与 `Display` 特征所约束，因此需要支持上述两种打印方式
- 有 `backtrace` 与 `cause` 方法，提供用户获取导致错误发生的信息
- 支持在附加的上下文信息中包装信息
- 由于有 `Send` 与 `sync` 特征约束，因此 failure 可以非常容易的在线程中传递
- 由于有 `'static`，因此抽象的 `Fail` 特征可以被向下转换成更具体的类型

任何新实现的错误类型都应该实现 `Fail` 特征，以便集成到整个系统中，可以通过 `derive` 的方式来实现，也可以通过自己手动实现`Fail` 特征。

##### `cause` 方法

通常情况下，一个错误类型可能包含另一种隐藏的真正引起这个错误的类型，`cause` 方法就是用来暴露隐藏的错误类型，可以在循环中不断调用 `cause` 方法暴露底层的错误类型。

##### `backtrace` 方法

在 `failure` 中暴露的 `backtrace` 与 `Backtrace` 包中的方法不一样。

##### `context` 方法

通常所使用的库可能不会提供非常清晰的错误导致原因，对于自己所实现的 `failure` 则可以通过 `ResultExt` 特征加上一些额外的上下文。

##### `Deriving Fail`
可以通过 `Fail`、`Display`特征获得上面的这些方法，也可以通过特殊属性去生成 `Display` 的derive，这样就不需要自己实现。
```rust

#![allow(unused_variables)]
fn main() {
extern crate failure;
#[macro_use] extern crate failure_derive;

#[derive(Fail, Debug)]
#[fail(display = "An error occurred.")]
struct MyError;
}
```

### 错误类型

直接返回 `String` 作为错误：这种在除了打印错误信息而没有别的方法来处理的时候再使用这种错误

定制一个错误类型：


