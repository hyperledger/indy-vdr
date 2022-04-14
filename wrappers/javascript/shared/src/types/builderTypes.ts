export type PrepareTxnAuthorAgreementAcceptanceOptions = {
  text?: string
  version?: string
  taaDigest?: string
  accMechType: string
  time: number
}

export type RequestSetSignatureOptions = {
  requestHandle: number
  signature: number
  signatureLen: number
}
