namespace indy_vdr_dotnet
{
    public static class ErrorCodeConverter
    {
        /// <summary>
        /// Converts the value of <see cref="ErrorCode"/> to the corresponding string representation for the backend.
        /// </summary>
        /// <returns>Matching expression for each provided code to use in error messages.</returns>
        public static string ToErrorCodeString(this ErrorCode errorCode)
        {
            switch (errorCode) 
            {
                case ErrorCode.Success:
                    return "Success";
                case ErrorCode.Config:
                    return "Config";
                case ErrorCode.Connection:
                    return "Connection";
                case ErrorCode.FileSystem:
                    return "FileSystem";
                case ErrorCode.Input:
                    return "Input";
                case ErrorCode.Resource:
                    return "Resource";
                case ErrorCode.Unavailable:
                    return "Unavailable";
                case ErrorCode.Unexpected:
                    return "Unexpected";
                case ErrorCode.Incompatible:
                    return "Incompatible";
                case ErrorCode.PoolNoConsensus:
                    return "PoolNoConsensus";
                case ErrorCode.PoolRequestFailed:
                    return "PoolRequestFailed";
                case ErrorCode.PoolTimeout:
                    return "PoolTimeout";
                default:
                    return "Unknown error code";
            }
        }
    }

    /// <summary>
    /// The error codes defined in the backend.
    /// </summary>
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