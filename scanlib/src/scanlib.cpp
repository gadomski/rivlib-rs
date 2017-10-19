#include <cstdint>
#include <riegl/scanlib.hpp>

struct Scanlib {
  std::string last_error;
};

struct Inclination {
  double time;
  double roll;
  double pitch;

  Inclination(double time, double roll, double pitch)
      : time(time), roll(roll), pitch(pitch) {}
};

struct Inclinations {
  Inclinations(std::vector<Inclination> inclinations)
      : inclinations(inclinations) {}
  std::vector<Inclination> inclinations;
};

class InclinationPointcloud : public scanlib::pointcloud {
public:
  InclinationPointcloud(bool sync_to_pps) : scanlib::pointcloud(sync_to_pps) {}
  std::vector<Inclination> &&into_inclinations() {
    return std::move(m_inclinations);
  }

protected:
  void on_hk_incl(const scanlib::hk_incl<iterator_type> &arg) {
    scanlib::pointcloud::on_hk_incl(arg);
    m_inclinations.push_back(
        Inclination(time, arg.ROLL * 0.001, arg.PITCH * 0.001));
  }

private:
  std::vector<Inclination> m_inclinations;
};

extern "C" {
void scanlib_new(Scanlib **scanlib) { *scanlib = new Scanlib(); }

const char *scanlib_last_error(Scanlib *scanlib) {
  return scanlib->last_error.c_str();
}

void scanlib_drop(Scanlib *scanlib) { delete scanlib; }

uint32_t inclinations_from_path(Scanlib *scanlib, const char *path,
                                bool sync_to_pps,
                                Inclinations **const pointer) {
  try {
    std::shared_ptr<scanlib::basic_rconnection> rc;
    rc = scanlib::basic_rconnection::create(path);
    rc->open();

    scanlib::decoder_rxpmarker dec(rc);
    scanlib::buffer buf;

    InclinationPointcloud pointcloud(sync_to_pps);
    for (dec.get(buf); !dec.eoi(); dec.get(buf)) {
      pointcloud.dispatch(buf.begin(), buf.end());
    }

    rc->close();
    *pointer = new Inclinations(pointcloud.into_inclinations());
    return 0;
  } catch (std::exception &err) {
    scanlib->last_error = err.what();
    return 1;
  } catch (...) {
    scanlib->last_error = "unknown";
    return -1;
  }
}

void inclinations_pointer(Inclinations *inclinations,
                          Inclination **const data) {
  *data = inclinations->inclinations.data();
}

void inclinations_len(const Inclinations *inclinations, uint64_t *len) {
  *len = inclinations->inclinations.size();
}

void inclinations_drop(Inclination *const pointer) { delete pointer; }
}
