namespace indy_vdr_dotnet
{
    public static class ErrorCodeConverter
    {
        public static string ToErrorCodeString(this ErrorCode errorCode)
        {
            return errorCode switch
            {
                ErrorCode.Success => "Success",
                ErrorCode.Config => "Config",
                ErrorCode.Connection => "Connection",
                ErrorCode.FileSystem => "FileSystem",
                ErrorCode.Input => "Input",
                ErrorCode.Resource => "Resource",
                ErrorCode.Unavailable => "Unavailable",
                ErrorCode.Unexpected => "Unexpected",
                ErrorCode.Incompatible => "Incompatible",
                ErrorCode.PoolNoConsensus => "PoolNoConsensus",
                ErrorCode.PoolRequestFailed => "PoolRequestFailed",
                ErrorCode.PoolTimeout => "PoolTimeout",
                _ => "Unknown error code"
            };
        }
    }
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
