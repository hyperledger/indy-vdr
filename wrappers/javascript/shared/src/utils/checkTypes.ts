type ShapeOf<T> = {
  [Property in keyof T]: T[Property]
}

export type GenericMethods<Base extends { [method: string]: any }> = {
  [Property in keyof Base]: (
    // eslint-disable-next-line @typescript-eslint/ban-types
    ...args: Parameters<Base[Property]>[0] extends {} ? [options: {}] : []
  ) => ReturnType<Base[Property]>
}

export type AssertEqual<X extends ShapeOf<Y>, Y extends ShapeOf<X>> = never
