using FluentAssertions;
using indy_vdr_dotnet.libindy_vdr;
using NUnit.Framework;

namespace indy_vdr_dotnet_tests.libindy_vdr
{
    public class RequestTests
    {
        [Test]
        [TestCase(TestName = "PrepareTxnAuthorAgreementAcceptance")]
        public void PrepareTxnAuthorAgreementAcceptance()
        {
            //Arrange

            //Act
            string expected = "";
            string actual = Request.PrepareTxnAuthorAgreementAcceptance("acc_mech_type", 10u, "text", "version", "taa_digest").GetAwaiter().GetResult();

            //Assert
            actual.Should().Be(expected);
        }
    }
}
