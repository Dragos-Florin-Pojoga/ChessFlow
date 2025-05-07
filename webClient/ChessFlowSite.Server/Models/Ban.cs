using System.ComponentModel.DataAnnotations;

namespace ChessFlowSite.Server.Models
{
    public class Ban
    {
        [Key]
        public int Id { get; set; }
        public string BannedId { get; set; }
        public virtual ApplicationUser? Banned {  get; set; }

        public string IssuerId { get; set; }
        public virtual ApplicationUser? Issuer { get; set; }

        [Required(ErrorMessage = "Reason is required"),
         MinLength(1, ErrorMessage = "Reason must be at least 1 character"),
         MaxLength(511, ErrorMessage = "Content cannot be longer than 511 characters.")]
        public string Reason { get; set; }

        public int? ReportId { get; set; }
        public virtual Report? Report { get; set; }

        public bool Permanent { get; set; }

        public DateTime BannedAt { get; set; } = DateTime.UtcNow;
        public DateTime? BannedUntil { get; set; }

        public bool IsActive => !BannedUntil.HasValue || BannedUntil > DateTime.UtcNow;
    }
}
