using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace indy_vdr_dotnet
{
    public enum ErrorCode
    {
        Success = 0,
        Config = 1,
        Connection = 2,
        FileSystem = 3,
        Input = 4,
        Resource = 5,
        Unavailable = 6,
        Unexpected = 7,
        Incompatible = 8,
        PoolNoConsensus = 30,
        PoolRequestFailed = 31,
        PoolTimeout = 32,
    }
}
