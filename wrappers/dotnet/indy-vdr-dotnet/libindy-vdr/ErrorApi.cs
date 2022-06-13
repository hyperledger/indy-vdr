using System.Threading.Tasks;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class ErrorApi
    {
        public static Task<string> GetCurrentErrorAsync()
        {
            string result = "";
            NativeMethods.indy_vdr_get_current_error(ref result);
            return Task.FromResult(result);
        }
    }
}
