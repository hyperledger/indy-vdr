using System.Threading.Tasks;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class Request
    {
        public static Task<string> PrepareTxnAuthorAgreementAcceptance(string acc_mech_type, uint time, string text = null, string version = null, string taa_digest = null)
        {
            string result = "";
            NativeMethods.indy_vdr_prepare_txn_author_agreement_acceptance(text, version, taa_digest, acc_mech_type, time, ref result);
            return Task.FromResult(result);
        }

    }
}
