using Microsoft.AspNetCore.Identity;
using System.ComponentModel.DataAnnotations;

namespace ChessFlowSite.Server.Models
{
    public class ApplicationUser : IdentityUser
    {
        [MinLength(5, ErrorMessage ="Username must have at least 5 chracters")]
        [MaxLength(31, ErrorMessage = "Username can have at most 31 characters")]
        public string Name { get; set; }
        public int? Elo {  get; set; }
    }
}
