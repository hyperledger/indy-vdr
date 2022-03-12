export type Raw = Record<string, unknown>

export type Aml = Record<string, unknown>

export type PoolStatus = {
  mt_root: string
  mt_size: number
  nodes: string[]
}

type Transaction = {
  reqSignature: Record<string | number, unknown>
  txn: {
    data: {
      data: {
        alias: string
        blskey: string
        blskey_pop: string
        client_ip: string
        client_port: string
        node_ip: string
        node_port: number
        services: string[]
      }
      dest: string
    }
    metadata: {
      from: string
    }
    type: string
  }
  txnMetadata: {
    seqNo: number
    txnId: string
  }
  ver: string
}

export type Transactions = Transaction[]

type VerifierInfo = {
  client_addr: string
  node_addr: string
  public_key: string
  enc_key: string
  bls_key?: string
}

export type Verifiers = Record<string, VerifierInfo>

// TODO: what is this?
export type SubmitAction = string

// TODO: what is this?
export type SubmitRequest = string
