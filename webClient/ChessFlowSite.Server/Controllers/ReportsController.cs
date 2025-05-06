using ChessFlowSite.Server.Models;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Identity;
using Microsoft.AspNetCore.Mvc;
using ReactApp1.Server.Data;

namespace ChessFlowSite.Server.Controllers
{
    [Route("api/[controller]")]
    [ApiController]
    public class ReportsController : Controller
    {

        private readonly ApplicationDbContext _db;
        private readonly UserManager<ApplicationUser> _userManager;

        public ReportsController(ApplicationDbContext db, UserManager<ApplicationUser> userManager)
        {
            _db = db;
            _userManager = userManager;
        }
        [HttpPost("create")]
        [Authorize]
        public async Task<IActionResult> Create([FromBody] ReportModel model)
        {
            var reported =  _db.ApplicationUsers.FirstOrDefault(u => u.Name == model.ReportedName);
            var reportee = await _userManager.GetUserAsync(User);

            if (reportee.Name != model.ReporteeName || reported == null || reported == reportee) {
                return BadRequest(new { errors = new[] { new { code = "InvalidUsers", description = "Reported and/or reportee users are invalid" } } });
            }

            if (model.Reason.Length > 511) {
                ModelState.AddModelError("ContentTooLong", "Content cannot be longer than 511 characters.");
            }
            if (model.Reason.Length < 1) {
                ModelState.AddModelError("ContentTooShort", "Reason must be at least 1 character");
            }
            if (!ModelState.IsValid) {
                var errorList = ModelState.Where(ms => ms.Value.Errors.Count > 0).SelectMany(kvp => kvp.Value.Errors.Select(e => new
                    {
                        code = kvp.Key,
                        description = e.ErrorMessage
                    })).ToArray();
                return BadRequest(new { errors = errorList });
            }

                Report report = new Report();
            report.Reported = reported;
            report.Reportee = reportee;
            if (model.GameId != null) {
                var game = _db.Games.FirstOrDefault(g => g.Id == model.GameId);
                if (game != null) {
                    report.Game = game;
                }
            }
            report.Reason = model.Reason;
            _db.Reports.Add(report);
            await _db.SaveChangesAsync();
            return Ok();
        }
    }
}
public class ReportModel
{
    public string ReportedName{ get; set; }
    public string ReporteeName{ get; set; }
    public int? GameId { get; set; }
    public string Reason { get; set; }
}
