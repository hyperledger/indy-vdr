using Newtonsoft.Json;
using System;
using System.Collections.Generic;

namespace indy_vdr_dotnet
{
    public class IndyVdrException : Exception
    {
        internal IndyVdrException(string message) : base(message)
        {
        }

        internal IndyVdrException(string message, Exception inner) : base(message, inner)
        {
        }

        internal static IndyVdrException FromSdkError(string message)
        {
            string msg = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["message"];
            string errCode = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["code"];
            string extra = JsonConvert.DeserializeObject<Dictionary<string, string>>(message)["extra"];
            int errCodeInt;
            if (int.TryParse(errCode, out errCodeInt))
            {
                string text;
                switch (errCodeInt)
                {
                    case (int)ErrorCode.Success:
                        {
                            text = ErrorCode.Success.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.Config:
                        {
                            text = ErrorCode.Config.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.Connection:
                        {
                            text = ErrorCode.Connection.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.FileSystem:
                        {
                            text = ErrorCode.FileSystem.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.Input:
                        {
                            text = ErrorCode.Input.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.Resource:
                        {
                            text = ErrorCode.Resource.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.Unavailable:
                        {
                            text = ErrorCode.Unavailable.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.Unexpected:
                        {
                            text = ErrorCode.Unexpected.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.Incompatible:
                        {
                            text = ErrorCode.Incompatible.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.PoolNoConsensus:
                        {
                            text = ErrorCode.PoolNoConsensus.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.PoolRequestFailed:
                        {
                            text = ErrorCode.PoolRequestFailed.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    case (int)ErrorCode.PoolTimeout:
                        {
                            text = ErrorCode.PoolTimeout.ToString();
                            return new IndyVdrException($"ErrorCode '{errCode}:{text}' with extra: '{extra}' was received: {msg}.");
                        }
                    default: return new IndyVdrException($"An unknown error with the error code '{errCodeInt}' was returned by the SDK.");
                }
            }
            else return new IndyVdrException("An unknown error code was received.");
        }
    }
}
