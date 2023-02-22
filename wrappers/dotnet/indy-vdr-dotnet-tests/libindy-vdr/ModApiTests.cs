using FluentAssertions;
using indy_vdr_dotnet;
using indy_vdr_dotnet.libindy_vdr;
using Newtonsoft.Json;
using NUnit.Framework;
using System;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class ModApiTests
    {
        #region Tests for GetVersionAsync
        [Test, TestCase(TestName = "GetVersionAsync() returns a string that is not empty.")]
        public async Task GetVersion()
        {
            //Arrange

            //Act
            string actual = await ModApi.GetVersionAsync();

            //Assert
            actual.Should().NotBeEmpty();
        }
        #endregion

        #region Tests for SetConfigAsync
        [Test, TestCase(TestName = "SetConfigAsync() sets the pool config.")]
        public async Task SetConfig()
        {
            //Arrange
            string testConfigJson = JsonConvert.SerializeObject(new
            {
                protocol_version = "Node1_4",
                freshness_threshold = 1,
                ack_timeout = 10000,
                reply_timeout = 10000,
                conn_request_limit = 1L,
                conn_active_timeout = 10000,
                request_read_nodes = 1L,
                socks_proxy = "proxy1.intranet.company.com:1080"
            });

            //Act
            int errorCode = await ModApi.SetConfigAsync(testConfigJson);

            //Assert
            errorCode.Should().Be(0);
            _ = await ModApi.SetConfigAsync(JsonConvert.SerializeObject(new { }));
        }

        [Test, TestCase(TestName = "SetConfigAsync() call throws.")]
        public async Task SetConfigThrows()
        {
            //Arrange
            string testConfigJson = "";

            //Act
            Func<Task> func = async () => await ModApi.SetConfigAsync(testConfigJson);

            //Assert
            await func.Should().ThrowAsync<IndyVdrException>();
            _ = await ModApi.SetConfigAsync(JsonConvert.SerializeObject(new { }));
        }
        #endregion

        #region Tests for SetDefaultLoggerAsync
        [Test, TestCase(TestName = "SetDefaultLoggerAsync() does not throw an exception.")]
        public async Task SetDefaultLogger()
        {
            //Arrange

            //Act
            int errorCode = await ModApi.SetDefaultLoggerAsync();

            //Assert
            errorCode.Should().Be(0);
        }
        #endregion

        #region Tests for SetSocksProxyAsync
        [Test, TestCase(TestName = "SetSocksProxyAsync() sets the socks proxy.")]
        public async Task SetSocksProxy()
        {
            //Arrange
            string testSocksProxy = "proxy1.intranet.company.com:1080";

            //Act
            int errorCode = await ModApi.SetSocksProxyAsync(testSocksProxy);

            //Assert
            errorCode.Should().Be(0);
            _ = await ModApi.SetConfigAsync(JsonConvert.SerializeObject(new { }));
        }
        #endregion

        #region Tests for SetProtocolVersionAsync
        [Test, TestCase(TestName = "SetProtocolVersionAsync() sets the protocol version.")]
        public async Task SetProtocolVersion()
        {
            //Arrange

            //Act                       
            int errorCode = await ModApi.SetProtocolVersionAsync(2);

            //Assert
            errorCode.Should().Be(0);
        }
        #endregion
    }
}
