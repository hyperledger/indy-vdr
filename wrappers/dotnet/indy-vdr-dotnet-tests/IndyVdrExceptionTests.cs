using FluentAssertions;
using indy_vdr_dotnet;
using NUnit.Framework;
using System;
using System.Collections.Generic;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests
{
    public class IndyVdrExceptionTests
    {
        private static IEnumerable<TestCaseData> CreateErrorCodeCases()
        {
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "0" , "Success")
                .SetName("IndyVdrExceptions contains ErrorCode 'Success' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "1", "Config")
                .SetName("IndyVdrExceptions contains ErrorCode 'Config' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "2", "Connection")
                .SetName("IndyVdrExceptions contains ErrorCode 'Connection' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "3", "FileSystem")
                .SetName("IndyVdrExceptions contains ErrorCode 'FileSystem' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "4", "Input")
                .SetName("IndyVdrExceptions contains ErrorCode 'Input' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "5", "Resource")
                .SetName("IndyVdrExceptions contains ErrorCode 'Resource' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "6", "Unavailable")
                .SetName("IndyVdrExceptions contains ErrorCode 'Unavailable' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "7", "Unexpected")
                .SetName("IndyVdrExceptions contains ErrorCode 'Unexpected' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "8", "Incompatible")
                .SetName("IndyVdrExceptions contains ErrorCode 'Incompatible' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "30", "PoolNoConsensus")
                .SetName("IndyVdrExceptions contains ErrorCode 'PoolNoConsensus' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "31", "PoolRequestFailed")
                .SetName("IndyVdrExceptions contains ErrorCode 'PoolRequestFailed' text after parsing the code to string.");
            yield return new TestCaseData("message matching to rust errorCode", "some exta from rust errorCode", "32", "PoolTimeout")
                .SetName("IndyVdrExceptions contains ErrorCode 'PoolTimeout' text after parsing the code to string.");
            yield return new TestCaseData("no message", "no extra", "99", "Unknown error code")
                .SetName("IndyVdrExceptions contains 'Unknown error code' text after trying to parse an unknown errorCode.");
            yield return new TestCaseData("no message", "no extra", "xyz", "An unknown error code was received.")
                .SetName("IndyVdrExceptions contains 'An unknown error code was received' text after trying to parse an non integer errorCode.");
        }

        [Test, TestCaseSource(nameof(CreateErrorCodeCases))]
        public async Task IndyVdrExceptionsRightMessages(string testMessage, string testExtra, string errorCode, string expected)
        {
            //Arrange
            string testErrorMessage = $"{{\"code\":\"{errorCode}\",\"message\":\"{testMessage}\",\"extra\":\"{testExtra}\" }}";

            //Act
            IndyVdrException testException = IndyVdrException.FromSdkError(testErrorMessage);
            string actual;
            if (errorCode != "xyz")
                actual = testException.Message.Substring(1, expected.Length);
            else
                actual = testException.Message;

            //Assert
            actual.Should().Be(expected);
            Console.WriteLine(testException.Message);
        }
    }
}
