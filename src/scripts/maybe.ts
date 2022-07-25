export type Maybe<T> = {
    value: T;
};

export function run<T>(input: Maybe<T>, transform: (_: T) => T): Maybe<T> {
    if (input === null) {
        return null;
    }
    return { value: transform(input.value) };
}
