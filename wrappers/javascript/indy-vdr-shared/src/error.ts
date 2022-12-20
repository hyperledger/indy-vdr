export type IndyVdrErrorObject = {
  code: number
  extra?: string
  message: string
}

export class IndyVdrError extends Error {
  public readonly code: number
  public readonly extra?: string

  public constructor({ code, message, extra }: IndyVdrErrorObject) {
    super(message)
    this.code = code
    this.extra = extra
  }
}
