using Swashbuckle.AspNetCore.Filters;

namespace ChessFlowSite.Server.Swagger.Examples
{
    public class AuthModelExample : IExamplesProvider<AuthModel>
    {
        public AuthModel GetExamples()
        {
            return new AuthModel
            {
                Email = "user@example.com",
                Password = "SecurePass123"
            };
        }
    }
}
