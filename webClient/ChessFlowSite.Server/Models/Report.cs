using System.ComponentModel.DataAnnotations;

namespace ChessFlowSite.Server.Models
{
    public class Report
    {
        [Key]
        public int Id { get; set; }
        [Required(ErrorMessage = "Reported is required")]
        public string ReporterId { get; set; }
        public virtual ApplicationUser? Reporter { get; set; }
        [Required(ErrorMessage = "Reportee is required")]
        public string ReporteeId { get; set; }
        public virtual ApplicationUser? Reportee {  get; set; }

        public int? GameId { get; set; }
        public virtual Game? Game { get; set; }

        [Required(ErrorMessage = "Reason is required"),
         MinLength(1, ErrorMessage = "Reason must be at least 1 character"),
         MaxLength(511, ErrorMessage = "Content cannot be longer than 511 characters.")]
        public string Reason { get; set; }
    }
}
