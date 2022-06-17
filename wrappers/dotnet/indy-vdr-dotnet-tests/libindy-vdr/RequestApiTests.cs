using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using NUnit.Framework;
using System;
using System.Threading.Tasks;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class RequestApiTests
    {
        [Test]
        [TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance")]
        public async Task PrepareTxnAuthorAgreementAcceptanceWorks()
        {
            //Arrange
            string expected = "";
            string testAccMechType = "acc_mech_type";
            ulong testTime = (ulong) DateTimeOffset.Now.ToUnixTimeSeconds();
            string testText = "text";
            string testVersion = "version";
            string testTaaDigest = "taa_digest";
            //Act

            string actual = await RequestApi.PrepareTxnAuthorAgreementAcceptance(
                testAccMechType,
                testTime,
                testText,
                testVersion,
                testTaaDigest);

            //Assert
            actual.Should().NotBe(expected);
        }
    }
}
