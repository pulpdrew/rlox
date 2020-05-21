#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OpCode {
    /// Loads the constant at `constants[arg1]` and leaves it at
    /// the top of the stack.
    Constant(usize),

    /// Sets the value at `stack[base]` to the value at `stack[top]`
    /// then ends execution of the current closure
    Return,

    /// Consume the two values at the top of the stack and leave
    /// `stack[top - 1]` + `stack[top]` in their place. Requires
    /// that both values are strings or both values are numbers.
    Add,

    /// Consume the two values at the top of the stack and leave
    /// `stack[top - 1]` - `stack[top]` in their place. Requires
    /// that both values are numbers.
    Subtract,

    /// Consume the two values at the top of the stack and leave
    /// `stack[top - 1]` * `stack[top]` in their place. Requires
    /// that both values are numbers.
    Multiply,

    /// Consume the two values at the top of the stack and leave
    /// `stack[top - 1]` / `stack[top]` in their place. Requires
    /// that both values are numbers.
    Divide,

    /// Consume the value at the top of the stack and leave its
    /// negation in its place. Requires that the value is a number.
    Negate,

    /// Consume the two values at the top of the stack and leave
    /// `Bool(stack[top - 1] < stack[top])` in their place. Requires
    /// that both values are numbers.
    Less,

    /// Consume the two values at the top of the stack and leave
    /// `Bool(stack[top - 1] > stack[top])` in their place. Requires
    /// that both values are numbers.
    Greater,

    /// Consume the two values at the top of the stack and leave
    /// `Bool(stack[top - 1] <= stack[top])` in their place. Requires
    /// that both values are numbers.
    LessEqual,

    /// Consume the two values at the top of the stack and leave
    /// `Bool(stack[top - 1] >= stack[top])` in their place. Requires
    /// that both values are numbers.
    GreaterEqual,

    /// Consume the value at the top of the stack and leave
    /// `!Truthiness(stack[top])` in its place.
    Not,

    /// Consume the two values at the top of the stack and leave
    /// `Bool(stack[top - 1] == stack[top])` in their place.
    Equal,

    /// Consume the two values at the top of the stack and leave
    /// `Bool(stack[top - 1] != stack[top])` in their place.
    NotEqual,

    /// Consume the value at the top of the stack and print it
    Print,

    /// Pops a single value from the stack and discards it
    Pop,

    /// Declares a new global variable with name `constants[arg1]`
    /// and value Nil
    DeclareGlobal(usize),

    /// Loads the value of the global with name `constants[arg1]`
    /// and leaves it on the stack
    GetGlobal(usize),

    /// Sets the value of the global with name `constants[arg1]`
    /// to the value at the top of the stack. Does not consume
    /// the value from the stack.
    SetGlobal(usize),

    /// Loads the value of the local at index `arg1` and leaves
    /// it on the top of the stack
    GetLocal(usize),

    /// Sets the value of the local at index `arg1` to the value
    /// at the top of the stack. Does not consume the value.
    SetLocal(usize),

    /// Looks up the method named `constants[arg1]` in the class
    /// at `stack[top]`, binds that method to the receiver at
    /// `stack[top -1]`, consumes the top 2 values on the stack,
    /// and leaves the bound method in their place.
    GetSuper(usize),

    /// Sets the `IP` to the `arg1`
    Jump(usize),

    /// Sets the `IP` to `arg1` if the value at the top of the
    /// stack is truthy. Does not consume the value.
    JumpIfTrue(usize),

    /// Sets the `IP` to `arg1` if the value at the top of the
    /// stack is not truthy. Does not consume the value.
    JumpIfFalse(usize),

    /// Calls the Value at `stack[top - arg1 - 1]` with the arguments
    /// `stack[top - arg1] .. stack[top]`. The called value must be
    /// a Closure, a Bound Method, or a Class.
    ///
    /// The value resulting from the call will be left on the top of
    /// the stack after everything from the callable up is consumed.
    ///
    /// Stack: [Callable] [Receiver] [arg] [arg] ... [arg]
    ///
    /// Closure: the closure's method is executed
    /// Bound Method: the method is executed with `this` = `stack[top - arg1]`
    /// Class: the class is instantiated
    Invoke(usize),

    /// Looks up the function `constants[arg1]`, then creates a closure
    /// from that function and the current values of all of its captured
    /// variables. The closure is left at the top of the stack.
    Closure(usize),

    /// Loads and pushes the upvalue at index `arg1` in the currently
    /// executing closure
    GetUpvalue(usize),

    /// Sets the value of the upvalue at index `arg1` in the currently
    /// executing closure to the value at the top of the stack. Does
    /// not consume the value at the top of the stack.
    SetUpvalue(usize),

    /// Loads the value of the field with the name `constants[arg1]`
    /// from the instance at the top of the stack. The instance is
    /// consumed and the field value is left in its place.
    ReadField(usize),

    /// Sets the values of the field with the name `constants[arg1]`
    /// from the instance at `stack[top-1]` to the value at the top
    /// of the stack. The instance is removed from the stack, but
    /// the value remains.
    SetField(usize),

    /// Consume the closure at `stack[top]` and add it to the class
    /// at `stack[top - 1]` as a method
    Method,

    /// Copy the methods from the super class `stack[top - 1]` into
    /// the subclass at `stack[top]`
    Inherit,

    /// Consume the value at the top of the stack and leave in its
    /// place a Value::Bool representing its truthiness
    Bool,
}
