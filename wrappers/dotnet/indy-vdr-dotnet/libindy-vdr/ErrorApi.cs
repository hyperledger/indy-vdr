using System.Threading.Tasks;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class ErrorApi
    {
        /// <summary>
        /// Returns json <see cref="System.String"/> of the last thrown native error.
        /// </summary>
        /// <returns>Error json in form of <c>{"code":[int],"message":[string]}</c>.</returns>
        public static Task<string> GetCurrentErrorAsync()
        {
            string result = "";
            _ = NativeMethods.indy_vdr_get_current_error(ref result);
            return Task.FromResult(result);
        }
    }
}
