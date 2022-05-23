using System.Threading.Tasks;

namespace indy_vdr_dotnet.libindy_vdr
{
    public static class Mod
    {
        public static Task<string> GetVersionAsync()
        {
            string result = NativeMethods.indy_vdr_version();
            return Task.FromResult(result);
        }
    }
}