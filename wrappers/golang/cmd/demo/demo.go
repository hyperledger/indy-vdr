package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"

	"github.com/hyperledger/indy-vdr/wrappers/golang/crypto"
	"github.com/hyperledger/indy-vdr/wrappers/golang/identifiers"
	"github.com/hyperledger/indy-vdr/wrappers/golang/vdr"
)

func main() {

	switch len(os.Args) {
	case 3:
		writeDemoTest()
	default:
		readOnlyDemo()
	}
}

func readOnlyDemo() {
	genesisFile, err := http.Get("https://raw.githubusercontent.com/sovrin-foundation/sovrin/master/sovrin/pool_transactions_builder_genesis")
	if err != nil {
		log.Fatalln(err)
	}
	defer genesisFile.Body.Close()

	client, err := vdr.New(genesisFile.Body)
	if err != nil {
		log.Fatalln(err)
	}

	err = client.RefreshPool()
	if err != nil {
		log.Fatalln(err)
	}

	status, err := client.GetPoolStatus()
	if err != nil {
		log.Fatalln(err)
	}

	d, _ := json.MarshalIndent(status, " ", " ")
	fmt.Println(string(d))

	rply, err := client.GetNym("FzAaV9Waa1DccDa72qwg13")
	if err != nil {
		log.Fatalln(err)
	}

	fmt.Println(rply.Data)
}

func writeDemoTest() {
	genesis, err := os.Open(os.Args[1])
	if err != nil {
		log.Fatalln("unable to open genesis file", err)
	}
	var TrusteeSeed = os.Args[2]

	client, err := vdr.New(genesis)
	if err != nil {
		log.Fatalln(err)
	}

	err = client.RefreshPool()
	if err != nil {
		log.Fatalln(err)
	}

	status, err := client.GetPoolStatus()
	if err != nil {
		log.Fatalln(err)
	}

	d, _ := json.MarshalIndent(status, " ", " ")
	fmt.Println(string(d))

	did, keyPair, err := identifiers.CreateDID(&identifiers.MyDIDInfo{Seed: TrusteeSeed[0:32], Cid: true, MethodName: "sov"})
	if err != nil {
		log.Fatalln(err)
	}

	mysig := crypto.NewSigner(keyPair.RawPublicKey(), keyPair.RawPrivateKey())

	fmt.Println("Steward DID:", did.String())
	fmt.Println("Steward Verkey:", did.Verkey)
	fmt.Println("Steward Short Verkey:", did.AbbreviateVerkey())

	someRandomDID, someRandomKP, err := identifiers.CreateDID(&identifiers.MyDIDInfo{MethodName: "sov", Cid: true})
	if err != nil {
		log.Fatalln(err)
	}

	err = client.CreateNym(someRandomDID.DIDVal.DID, someRandomDID.Verkey, vdr.EndorserRole, did.DIDVal.DID, mysig)
	if err != nil {
		log.Fatalln(err)
	}
	fmt.Println("New Endorser DID:", someRandomDID.String())
	fmt.Println("New Endorser Verkey:", someRandomDID.AbbreviateVerkey())
	fmt.Println("Place These in Wallet:")
	fmt.Println("Public:", someRandomKP.PublicKey())
	fmt.Println("Private:", someRandomKP.PrivateKey())

	newDIDsig := crypto.NewSigner(someRandomKP.RawPublicKey(), someRandomKP.RawPrivateKey())

	err = client.SetEndpoint(someRandomDID.DIDVal.DID, someRandomDID.DIDVal.DID, "http://420.69.420.69:6969", newDIDsig)
	if err != nil {
		log.Fatalln(err)
	}

	rply, err := client.GetNym(someRandomDID.DIDVal.DID)
	if err != nil {
		log.Fatalln(err)
	}

	fmt.Println(rply.Data)

	rply, err = client.GetEndpoint(someRandomDID.DIDVal.DID)
	if err != nil {
		log.Fatalln(err)
	}

	d, _ = json.MarshalIndent(rply, " ", " ")
	fmt.Println(string(d))

	//	rply, err = client.GetAuthRules()
	rply, err = client.GetTxnTypeAuthRule("1", "EDIT", "role")
	if err != nil {
		log.Fatalln(err)
	}

	d, _ = json.MarshalIndent(rply, " ", " ")
	fmt.Println(string(d))

	//rply, err = client.GetCredDef("Xy9dvEi8dkkPif5j342w9q:3:CL:23:default")
	//if err != nil {
	//	log.Fatalln(err)
	//}
	//
	//d, _ = json.MarshalIndent(rply, " ", " ")
	//fmt.Println(string(d))

	//rply, err = client.GetSchema("Xy9dvEi8dkkPif5j342w9q:2:Scoir High School:0.0.1")
	//if err != nil {
	//	log.Fatalln(err)
	//}
	//
	//d, _ = json.MarshalIndent(rply, " ", " ")
	//fmt.Println(string(d))
	//

}
