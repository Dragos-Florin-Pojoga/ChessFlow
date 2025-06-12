using Swashbuckle.AspNetCore.Filters;

namespace ChessFlowSite.Server.Swagger.Examples
{
    public class RegModelExample : IExamplesProvider<RegModel>
    {
        public RegModel GetExamples()
        {
            return new RegModel
            {
                Email = "newuser@example.com",
                Password = "StrongPass!456",
                Name = "ChessMaster99",
                Elo = 1200
            };
        }
    }
}
