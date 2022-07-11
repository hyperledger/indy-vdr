using System;
using System.Threading.Tasks;
using static indy_vdr_dotnet.models.Structures;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class RequestApi
    {
        /// <summary>
        /// Creates a new TAA from the provided parameters.
        /// 
        /// Either 'text' + 'version' or 'taa_digest' have to be passed (not both).
        /// </summary>
        /// <param name="accMechType">Mechanism type.</param>
        /// <param name="time">Unix timestamp</param>
        /// <param name="text"></param>
        /// <param name="version">TAA version.</param>
        /// <param name="taaDigest">TAA digest.</param>
        /// <exception cref="IndyVdrException">Throws if parameters or the combination of them are invalid.</exception>
        /// <returns>The transaction author agreement acceptance as json <see cref="System.String"/>.</returns>
        public static async Task<string> PrepareTxnAuthorAgreementAcceptanceAsync(
            string accMechType,
            ulong time,
            string text = null,
            string version = null,
            string taaDigest = null)
        {
            string agreementAcceptance = "";
            int errorCode = NativeMethods.indy_vdr_prepare_txn_author_agreement_acceptance(
                FfiStr.Create(text),
                FfiStr.Create(version),
                FfiStr.Create(taaDigest),
                FfiStr.Create(accMechType),
                time,
                ref agreementAcceptance);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }

            return agreementAcceptance;
        }

        /// <summary>
        /// Frees a request object from the heap.
        /// </summary>
        /// <param name="requestHandle">Handle of the request object.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="requestHandle"/> is invalid.</exception>
        public static async Task RequestFreeAsync(
            IntPtr requestHandle)
        {
            int errorCode = NativeMethods.indy_vdr_request_free(
                requestHandle);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        /// <summary>
        /// Gets the body of a request from its handle.
        /// </summary>
        /// <param name="requestHandle">Handle of the request object.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="requestHandle"/> is invalid.</exception>
        /// <returns>Body of the request as json <see cref="System.String"/>.</returns>
        public static async Task<string> RequestGetBodyAsync(
            IntPtr requestHandle)
        {
            string body = "";
            int errorCode = NativeMethods.indy_vdr_request_get_body(
                requestHandle,
                ref body);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return body;
        }

        /// <summary>
        /// Get the signature input of this prepared request.
        /// 
        /// This value should be passed to the signer in order to generate an
        /// appropriate signature for the request.
        /// </summary>
        /// <param name="requestHandle">Handle of the request object.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="requestHandle"/> is invalid.</exception>
        /// <returns>The values of the request signature seperated by the character '<c>|</c>'.</returns>
        public static async Task<string> RequestGetSignatureInputAsync(
            IntPtr requestHandle)
        {
            string signature = "";
            int errorCode = NativeMethods.indy_vdr_request_get_signature_input(
                requestHandle,
                ref signature);

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
            return signature;
        }

        /// <summary>
        /// Set the endorser property of a provideds prepared request.
        ///
        /// When a transaction is expected to be sent to the ledger by an endorser, this
        /// property must be set in order to generate a valid signature.
        /// </summary>
        /// <remarks>
        /// Note: Both Transaction Author and Endorser must sign the output request.
        /// More about Transaction Endorser:
        ///    <c>https://github.com/hyperledger/indy-node/blob/master/design/transaction_endorser.md</c>, 
        ///    <c>https://github.com/hyperledger/indy-sdk/blob/master/docs/configuration.md</c>
        /// </remarks>
        /// <param name="requestHandle">Handle of a prepared request object.</param>
        /// <param name="endorser">DID of the Endorser that will submit the transaction. The Endorser's DID must be present on the ledger.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="requestHandle"/> ir <paramref name="endorser"/> are invalid.</exception>
        public static async Task RequestSetEndorserAsync(
            IntPtr requestHandle,
            string endorser)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_endorser(
                requestHandle,
                FfiStr.Create(endorser));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        /// <summary>
        /// Add a multi-signature entry to a provided prepared request.
        /// </summary>
        /// <param name="requestHandle">Handle of a prepared request object.</param>
        /// <param name="identifier">The DID of the signer.</param>
        /// <param name="signature">The ed25519 signature.</param>
        /// <exception cref="IndyVdrException">Throws if any paramter is invalid.</exception>
        public static async Task RequestSetMultiSignatureAsync(
            IntPtr requestHandle,
            string identifier,
            string signature)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_multi_signature(
                requestHandle,
                FfiStr.Create(identifier),
                ByteBuffer.Create(signature));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        /// <summary>
        /// Set the signature on a prepared request.
        /// </summary>
        /// <param name="requestHandle">Handle of a prepared request object.</param>
        /// <param name="signature">The ed25519 signature.</param>
        /// <exception cref="IndyVdrException">Throws if <paramref name="requestHandle"/> or <paramref name="signature"/> is invalid.</exception>
        public static async Task RequestSetSigantureAsync(
            IntPtr requestHandle,
            string signature)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_signature(
                requestHandle,
                ByteBuffer.Create(signature));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }

        /// <summary>
        /// Set the TAA acceptance on a prepared request.
        /// </summary>
        /// <param name="requestHandle">Handle of a prepared request object.</param>
        /// <param name="agreementAcceptance">TAA to set.</param>
        /// <exception cref="IndyVdrException">Throws if any parameter is invalid.</exception>
        public static async Task RequestSetTxnAuthorAgreementAcceptanceAsync(
            IntPtr requestHandle,
            string agreementAcceptance)
        {
            int errorCode = NativeMethods.indy_vdr_request_set_txn_author_agreement_acceptance(
                requestHandle,
                FfiStr.Create(agreementAcceptance));

            if (errorCode != (int)ErrorCode.Success)
            {
                string error = await ErrorApi.GetCurrentErrorAsync();
                throw IndyVdrException.FromSdkError(error);
            }
        }
    }
}
