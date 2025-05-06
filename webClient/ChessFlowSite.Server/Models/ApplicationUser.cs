using Microsoft.AspNetCore.Identity;

namespace ChessFlowSite.Server.Models
{
    public class ApplicationUser : IdentityUser
    {
        public int? Elo {  get; set; }
    }
}
